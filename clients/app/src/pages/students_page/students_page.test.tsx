import { expect, open, test } from "nsh/test";

test("students page displays aligned roster, filters, and supports compact layout", async () => {
  const desktop = await open("../../..", { width: 1200, height: 800, platform: "macos" });
  await desktop.press("role:button[管理员]");
  await desktop.waitFor("学号");
  expect(desktop.texts()).toContain("20261001");
  expect(desktop.texts()).toContain("张晓明");
  expect(desktop.texts()).toContain("高一 1 班");

  await desktop.type("text:搜索学号、姓名或班级", "王梓轩");
  await desktop.waitFor("王梓轩");
  expect(desktop.shows("张晓明")).toBe(false);

  const compact = await open("../../..", { width: 390, height: 844, platform: "macos" });
  await compact.press("role:button[管理员]");
  await compact.waitFor("20261001");
  expect(compact.shows("role:button[退出]")).toBe(true);
});

test("admin enrolls student via modal with capacity guard while teacher has no enroll action", async () => {
  const teacher = await open("../../..", { width: 1200, height: 800, platform: "macos" });
  await teacher.press("role:button[教师]");
  await teacher.waitFor("20261001");
  expect(teacher.shows("role:button[办理入学…]")).toBe(false);

  const admin = await open("../../..", { width: 1200, height: 800, platform: "macos" });
  await admin.press("role:button[管理员]");
  await admin.waitFor("role:button[办理入学…]");

  await admin.press("role:button[办理入学…]");
  await admin.waitFor("新生入学建档");

  await admin.type("text:请输入学生姓名", "周子昂");
  await admin.press("role:combobox[目标行政班]");
  await admin.waitFor("高一 2 班");
  await admin.press("高一 2 班");
  await admin.press("role:button[确认入学]");
  await admin.waitFor("text:/额定/");

  await admin.press("role:combobox[目标行政班]");
  await admin.waitFor("高一 1 班");
  await admin.press("高一 1 班");
  await admin.press("role:button[确认入学]");
  await admin.waitFor("20261007");
  expect(admin.texts()).toContain("周子昂");
});
