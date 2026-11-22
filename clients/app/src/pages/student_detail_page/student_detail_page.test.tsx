import { expect, open, test } from "nsh/test";

test("drilling into student detail shows enrollment history and pops back", async () => {
  const app = await open("../../..", { width: 1200, height: 800, platform: "macos" });
  await app.type("text:请输入登录账号", "admin");
  await app.type("text:输入登录密码", "demo");
  await app.press("role:button[登录]");
  await app.waitFor("张晓明");

  await app.press("张晓明");
  await app.waitFor("学籍异动");
  expect(app.shows("text:/20261001/")).toBe(true);
  expect(app.shows("text:/入学建档/")).toBe(true);

  await app.press("role:button[返回]");
  await app.waitFor("20261002");
});

test("admin executes transfer, suspend, resume, withdraw, and terminal reversal", async () => {
  const app = await open("../../..", { width: 1200, height: 800, platform: "macos" });
  await app.type("text:请输入登录账号", "admin");
  await app.type("text:输入登录密码", "demo");
  await app.press("role:button[登录]");
  await app.waitFor("张晓明");

  await app.press("张晓明");
  await app.waitFor("role:button[转班…]");

  await app.press("role:button[转班…]");
  await app.waitFor("行政转班");
  await app.press("role:combobox[目标行政班]");
  await app.waitFor("高二 1 班");
  await app.press("高二 1 班");
  await app.type("text:请输入异动原因", "走读调整");
  await app.press("role:button[确认提交]");
  await app.waitFor("text:/行政转班/");
  expect(app.shows("text:/高二 1 班/")).toBe(true);

  await app.press("role:button[休学…]");
  await app.waitFor("休学登记");
  await app.type("text:请输入异动原因", "因病休学");
  await app.press("role:button[确认提交]");
  await app.waitFor("role:button[复学…]");

  await app.press("role:button[复学…]");
  await app.waitFor("复学登记");
  await app.type("text:请输入异动原因", "康复返校");
  await app.press("role:button[确认提交]");
  await app.waitFor("role:button[毕业…]");

  await app.press("role:button[退学…]");
  await app.waitFor("退学登记");
  await app.type("text:请输入异动原因", "主动退学");
  await app.press("role:button[确认提交]");
  await app.waitFor("role:button[冲正…]");

  await app.press("role:button[冲正…]");
  await app.waitFor("终态冲正");
  await app.type("text:请输入异动原因", "误操作更正");
  await app.press("role:button[确认提交]");
  await app.waitFor("role:button[转班…]");
});
