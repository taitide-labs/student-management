import { expect, open, test } from "nsh/test";

test("teacher saves draft and publishes grades while admin can amend published grade", async () => {
  const teacher = await open("../../..", { width: 1200, height: 800, platform: "macos" });
  await teacher.type("text:请输入登录账号", "teacher");
  await teacher.type("text:输入登录密码", "demo");
  await teacher.press("role:button[登录]");
  await teacher.waitFor("role:button[成绩]");
  await teacher.press("role:button[成绩]");
  await teacher.waitFor("role:button[录入成绩…]");

  await teacher.press("role:button[录入成绩…]");
  await teacher.waitFor("保存成绩草稿");
  await teacher.type("text:请输入学生学号", "20261001");
  await teacher.type("text:请输入分值", "91.5");
  await teacher.press("role:button[保存草稿]");
  await teacher.waitFor("91.5");

  await teacher.press("role:button[锁分发布]");
  await teacher.waitFor("已锁定");
  expect(teacher.shows("role:button[修订…]")).toBe(false);

  const admin = await open("../../..", { width: 1200, height: 800, platform: "macos" });
  await admin.type("text:请输入登录账号", "admin");
  await admin.type("text:输入登录密码", "demo");
  await admin.press("role:button[登录]");
  await admin.waitFor("role:button[成绩]");
  await admin.press("role:button[成绩]");
  await admin.waitFor("text:/MATH101/");
  await admin.press("role:button[修订…]");
  await admin.waitFor("合规修订成绩");
  await admin.type("text:请输入修订后分值", "95");
  await admin.type("text:请输入修订原因", "试卷复核加分");
  await admin.press("role:button[确认修订]");
  await admin.waitFor("95");
});
