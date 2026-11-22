import { expect, open, test } from "nsh/test";

test("login screen authenticates admin into workspace without in-app role switcher", async () => {
  const app = await open("../../..", { width: 1200, height: 800, platform: "macos" });
  await app.waitFor("学生管理系统");
  expect(app.shows("role:button[登录]")).toBe(true);

  await app.press("role:button[管理员]");
  await app.waitFor("学生");
  expect(app.shows("演示身份切换")).toBe(false);
  expect(app.shows("模拟断网")).toBe(false);

  await app.press("role:button[退出]");
  await app.waitFor("role:button[登录]");
});
