import { expect, open, test } from "nsh/test";

test("admin creates homeroom and updates capacity with active count guard", async () => {
  const app = await open("../../..", { width: 1200, height: 800, platform: "macos" });
  await app.type("text:请输入登录账号", "admin");
  await app.type("text:输入登录密码", "demo");
  await app.press("role:button[登录]");
  await app.waitFor("role:button[行政班]");
  await app.press("role:button[行政班]");
  await app.waitFor("role:button[新建行政班…]");

  await app.press("role:button[新建行政班…]");
  await app.waitFor("创建行政班");
  await app.type("text:请输入班级名称", "高一 3 班");
  await app.type("text:请输入入学年份", "2026");
  await app.type("text:可选额定人数", "40");
  await app.press("role:button[确认创建]");
  await app.waitFor("text:/^已创建行政班/");
  await app.waitFor("高一 3 班");

  await app.press("role:button[调整高一 1 班容量…]");
  await app.waitFor("调整额定容量");
  await app.type("text:留空表示不限班额", "1");
  await app.press("role:button[确认调整]");
  await app.waitFor("text:/低于当前在籍人数/");
});
