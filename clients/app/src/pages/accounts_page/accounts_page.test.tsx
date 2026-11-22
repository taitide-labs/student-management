import { expect, open, test } from "nsh/test";

test("admin provisions account, changes role, and disables account", async () => {
  const app = await open("../../..", { width: 1200, height: 800, platform: "macos" });
  await app.press("role:button[管理员]");
  await app.waitFor("role:button[账号]");
  await app.press("role:button[账号]");
  await app.waitFor("role:button[开通账号…]");

  await app.press("role:button[开通账号…]");
  await app.waitFor("开通登录账号");
  await app.type("text:请输入登录用户名", "wanglaoshi");
  await app.type("text:请输入显示姓名", "王老师");
  await app.press("role:button[确认开通]");
  await app.waitFor("王老师");

  await app.press("role:button[停用王老师]");
  await app.waitFor("停用账号");
  await app.type("text:请输入停用原因", "岗位调整");
  await app.press("role:button[确认停用]");
  await app.waitFor("已停用");
});
