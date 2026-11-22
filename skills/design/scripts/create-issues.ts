#!/usr/bin/env bun

import { readFileSync } from "node:fs";

interface IssueSpec {
  id: string;
  title: string;
  body?: string;
  parent?: string;
  blocked_by?: string[];
  labels?: string[];
  type?: string;
}

interface IssuePlan {
  milestone?: string;
  issues: IssueSpec[];
}

interface CliOptions {
  dryRun: boolean;
  repo?: string;
  planPath: string;
}

const USAGE = `Usage: create-issues.ts [--dry-run] [--repo [HOST/]OWNER/REPO] <plan.yaml>

Options:
  --dry-run                   Validate plan, check cycles, and print execution order without calling GitHub
  --repo [HOST/]OWNER/REPO    Target GitHub repository (defaults to current git repo)
  -h, --help                  Show this help message

YAML schema:
  milestone: v0.1.0               # optional; auto-created if missing
  issues:
    - id: checkout-flow           # unique local identifier
      title: "Issue title"
      body: |
        Markdown body
      parent: existing-id-or-123  # optional parent id in plan or existing issue number
      blocked_by: [id-a, 42]      # optional list of plan ids or existing issue numbers
      labels: [enhancement]       # optional
      type: Task                  # optional
`;

function fail(message: string): never {
  console.error(`Error: ${message}`);
  process.exit(1);
}

function parseArgs(argv: string[]): CliOptions {
  let dryRun = false;
  let repo: string | undefined;
  let planPath: string | undefined;

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "-h" || arg === "--help") {
      console.log(USAGE);
      process.exit(0);
    }
    if (arg === "--dry-run") {
      dryRun = true;
      continue;
    }
    if (arg === "--repo" || arg === "-R") {
      const next = argv[i + 1];
      if (!next) {
        fail(`${arg} requires a [HOST/]OWNER/REPO value`);
      }
      repo = next.trim();
      i += 1;
      continue;
    }
    if (arg.startsWith("-")) {
      fail(`Unknown option: ${arg}`);
    }
    if (planPath) {
      fail(`Unexpected extra argument: ${arg}`);
    }
    planPath = arg;
  }

  if (!planPath) {
    console.error(USAGE);
    process.exit(1);
  }

  return { dryRun, repo, planPath };
}

function parseExternalIssueNumber(ref: string | number): string | null {
  if (typeof ref === "number") {
    return Number.isInteger(ref) && ref > 0 ? String(ref) : null;
  }
  const match = /^#?([1-9]\d*)$/.exec(ref.trim());
  return match ? match[1] : null;
}

function normalizeRef(ref: string | number, localIds: Set<string>, ownerId: string, field: string): string {
  if (typeof ref === "string") {
    const trimmed = ref.trim();
    if (localIds.has(trimmed)) {
      if (trimmed === ownerId) {
        fail(`Issue "${ownerId}" cannot reference itself in ${field}`);
      }
      return trimmed;
    }
  }
  const external = parseExternalIssueNumber(ref);
  if (external !== null) {
    return external;
  }
  fail(`Issue "${ownerId}" references unknown ${field}: "${String(ref)}"`);
}

function loadAndValidatePlan(planPath: string): { plan: IssuePlan; ordered: IssueSpec[] } {
  let rawText: string;
  try {
    rawText = readFileSync(planPath, "utf8");
  } catch (error) {
    fail(`Cannot read plan file "${planPath}": ${error instanceof Error ? error.message : String(error)}`);
  }

  let parsed: unknown;
  try {
    parsed = Bun.YAML.parse(rawText);
  } catch (error) {
    fail(`Invalid YAML in "${planPath}": ${error instanceof Error ? error.message : String(error)}`);
  }

  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    fail("Plan root must be a YAML mapping");
  }

  const record = parsed as Record<string, unknown>;
  if (record.milestone !== undefined && (typeof record.milestone !== "string" || !record.milestone.trim())) {
    fail("milestone must be a non-empty string");
  }
  if (!Array.isArray(record.issues) || record.issues.length === 0) {
    fail("issues must be a non-empty list");
  }

  interface RawIssue {
    id: string;
    title: string;
    body?: string;
    parent?: string | number;
    blocked_by?: Array<string | number>;
    labels?: string[];
    type?: string;
  }

  const rawSpecs: RawIssue[] = [];
  const localIds = new Set<string>();

  for (let i = 0; i < record.issues.length; i += 1) {
    const entry = record.issues[i];
    if (!entry || typeof entry !== "object" || Array.isArray(entry)) {
      fail(`issues[${i}] must be a mapping`);
    }
    const item = entry as Record<string, unknown>;
    if (typeof item.id !== "string" || !item.id.trim()) {
      fail(`issues[${i}].id must be a non-empty string`);
    }
    const id = item.id.trim();
    if (localIds.has(id)) {
      fail(`Duplicate issue id: "${id}"`);
    }
    if (typeof item.title !== "string" || !item.title.trim()) {
      fail(`issues[${i}] ("${id}") must have a non-empty title`);
    }
    if (item.body !== undefined && typeof item.body !== "string") {
      fail(`issues["${id}"].body must be a string`);
    }
    if (item.parent !== undefined && typeof item.parent !== "string" && typeof item.parent !== "number") {
      fail(`issues["${id}"].parent must be a string or issue number`);
    }
    if (item.blocked_by !== undefined) {
      if (
        !Array.isArray(item.blocked_by) ||
        item.blocked_by.some((v) => typeof v !== "string" && typeof v !== "number")
      ) {
        fail(`issues["${id}"].blocked_by must be a list of ids or issue numbers`);
      }
    }
    let normalizedLabels: string[] | undefined;
    if (item.labels !== undefined) {
      if (!Array.isArray(item.labels) || item.labels.some((v) => typeof v !== "string" || !v.trim())) {
        fail(`issues["${id}"].labels must be a list of non-empty strings`);
      }
      normalizedLabels = Array.from(new Set((item.labels as string[]).map((label) => label.trim())));
    }
    if (item.type !== undefined && (typeof item.type !== "string" || !item.type.trim())) {
      fail(`issues["${id}"].type must be a non-empty string`);
    }

    localIds.add(id);
    rawSpecs.push({
      id,
      title: item.title.trim(),
      body: item.body,
      parent: item.parent as string | number | undefined,
      blocked_by: item.blocked_by as Array<string | number> | undefined,
      labels: normalizedLabels,
      type: item.type ? (item.type as string).trim() : undefined,
    });
  }

  const specs: IssueSpec[] = [];
  const byId = new Map<string, IssueSpec>();
  const inDegree = new Map<string, number>();
  const dependents = new Map<string, string[]>();

  for (const raw of rawSpecs) {
    const normalizedParent =
      raw.parent !== undefined ? normalizeRef(raw.parent, localIds, raw.id, "parent") : undefined;
    const normalizedBlockers: string[] = [];
    const seenBlockers = new Set<string>();
    for (const blocker of raw.blocked_by ?? []) {
      const normalized = normalizeRef(blocker, localIds, raw.id, "blocked_by");
      if (normalizedParent !== undefined && normalized === normalizedParent) {
        fail(`Issue "${raw.id}" cannot list its own parent "${normalized}" in blocked_by`);
      }
      if (!seenBlockers.has(normalized)) {
        seenBlockers.add(normalized);
        normalizedBlockers.push(normalized);
      }
    }

    const spec: IssueSpec = {
      id: raw.id,
      title: raw.title,
      body: raw.body,
      parent: normalizedParent,
      blocked_by: normalizedBlockers.length > 0 ? normalizedBlockers : undefined,
      labels: raw.labels,
      type: raw.type,
    };
    specs.push(spec);
    byId.set(spec.id, spec);
    inDegree.set(spec.id, 0);
    dependents.set(spec.id, []);
  }

  for (const spec of specs) {
    const prerequisites: string[] = [];
    if (spec.parent && localIds.has(spec.parent)) {
      prerequisites.push(spec.parent);
    }
    for (const blocker of spec.blocked_by ?? []) {
      if (localIds.has(blocker)) {
        prerequisites.push(blocker);
      }
    }
    for (const prereq of prerequisites) {
      dependents.get(prereq)!.push(spec.id);
      inDegree.set(spec.id, inDegree.get(spec.id)! + 1);
    }
  }

  const queue: string[] = [];
  for (const spec of specs) {
    if (inDegree.get(spec.id) === 0) {
      queue.push(spec.id);
    }
  }

  const ordered: IssueSpec[] = [];
  while (queue.length > 0) {
    const currentId = queue.shift()!;
    ordered.push(byId.get(currentId)!);
    for (const nextId of dependents.get(currentId)!) {
      const remaining = inDegree.get(nextId)! - 1;
      inDegree.set(nextId, remaining);
      if (remaining === 0) {
        queue.push(nextId);
      }
    }
  }

  if (ordered.length !== specs.length) {
    const cyclic = specs.filter((s) => (inDegree.get(s.id) ?? 0) > 0).map((s) => s.id);
    fail(`Dependency cycle detected involving: ${cyclic.join(", ")}`);
  }

  const plan: IssuePlan = {
    milestone: typeof record.milestone === "string" ? record.milestone.trim() : undefined,
    issues: specs,
  };
  return { plan, ordered };
}

function runGh(args: string[], stdinText?: string): string {
  const proc = Bun.spawnSync(["gh", ...args], {
    stdin: stdinText !== undefined ? Buffer.from(stdinText, "utf8") : "ignore",
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = proc.stdout.toString("utf8").trim();
  const stderr = proc.stderr.toString("utf8").trim();
  if (proc.exitCode !== 0) {
    fail(`gh ${args.join(" ")} failed (exit ${proc.exitCode}): ${stderr || stdout}`);
  }
  return stdout;
}

function parseRepoTarget(repoSlug: string): { host?: string; ownerRepo: string } {
  const parts = repoSlug.split("/").filter(Boolean);
  if (parts.length === 2) {
    return { ownerRepo: `${parts[0]}/${parts[1]}` };
  }
  if (parts.length === 3) {
    return { host: parts[0], ownerRepo: `${parts[1]}/${parts[2]}` };
  }
  fail(`Invalid repository format "${repoSlug}", expected OWNER/REPO or HOST/OWNER/REPO`);
}

function resolveRepoSlug(repo?: string): string {
  if (repo) {
    return repo;
  }
  return runGh(["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"]);
}

function ensureMilestone(repoSlug: string, milestone: string): void {
  const { host, ownerRepo } = parseRepoTarget(repoSlug);
  const hostArgs = host ? ["--hostname", host] : [];
  const existing = runGh([
    "api",
    ...hostArgs,
    `repos/${ownerRepo}/milestones?state=all`,
    "--paginate",
    "--jq",
    ".[].title",
  ]);
  const titles = existing ? existing.split("\n").map((line) => line.trim()) : [];
  if (titles.includes(milestone)) {
    return;
  }
  runGh(["api", ...hostArgs, "--method", "POST", `repos/${ownerRepo}/milestones`, "-f", `title=${milestone}`]);
  console.log(`Created milestone: ${milestone}`);
}

function resolveRef(ref: string, created: Map<string, string>, localIds: Set<string>, dryRun: boolean): string {
  if (localIds.has(ref)) {
    const mapped = created.get(ref);
    if (mapped) {
      return mapped;
    }
    if (dryRun) {
      return `<${ref}>`;
    }
    fail(`Unresolved local reference: "${ref}"`);
  }
  return ref;
}

function extractIssueNumber(issueUrl: string): string {
  const match = /\/issues\/(\d+)$/.exec(issueUrl.trim());
  if (!match) {
    fail(`Unable to parse issue number from gh output: "${issueUrl}"`);
  }
  return match[1];
}

function main(): void {
  const options = parseArgs(process.argv.slice(2));
  const { plan, ordered } = loadAndValidatePlan(options.planPath);
  const localIds = new Set(plan.issues.map((issue) => issue.id));

  if (options.dryRun) {
    if (plan.milestone) {
      console.log(`[dry-run] Ensure milestone: ${plan.milestone}`);
    }
    for (const spec of ordered) {
      const meta: string[] = [];
      if (spec.parent !== undefined) {
        meta.push(`parent=${resolveRef(spec.parent, new Map(), localIds, true)}`);
      }
      if (spec.blocked_by && spec.blocked_by.length > 0) {
        meta.push(
          `blocked_by=${spec.blocked_by.map((b) => resolveRef(b, new Map(), localIds, true)).join(",")}`,
        );
      }
      if (plan.milestone) {
        meta.push(`milestone=${plan.milestone}`);
      }
      if (spec.labels && spec.labels.length > 0) {
        meta.push(`labels=${spec.labels.join(",")}`);
      }
      if (spec.type) {
        meta.push(`type=${spec.type}`);
      }
      const suffix = meta.length > 0 ? ` (${meta.join("; ")})` : "";
      console.log(`[dry-run] ${spec.id}: ${spec.title}${suffix}`);
    }
    return;
  }

  const repoSlug = resolveRepoSlug(options.repo);
  if (plan.milestone) {
    ensureMilestone(repoSlug, plan.milestone);
  }

  const created = new Map<string, string>();
  for (const spec of ordered) {
    const args = ["issue", "create", "-R", repoSlug, "--title", spec.title, "--body-file", "-"];
    if (plan.milestone) {
      args.push("--milestone", plan.milestone);
    }
    if (spec.parent !== undefined) {
      args.push("--parent", resolveRef(spec.parent, created, localIds, false));
    }
    if (spec.blocked_by && spec.blocked_by.length > 0) {
      const blockers = spec.blocked_by.map((b) => resolveRef(b, created, localIds, false)).join(",");
      args.push("--blocked-by", blockers);
    }
    for (const label of spec.labels ?? []) {
      args.push("--label", label);
    }
    if (spec.type) {
      args.push("--type", spec.type);
    }

    const url = runGh(args, spec.body ?? "");
    const number = extractIssueNumber(url);
    created.set(spec.id, number);
    console.log(`${spec.id} -> #${number} ${url}`);
  }
}

main();
