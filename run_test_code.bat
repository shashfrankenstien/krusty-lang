@ECHO OFF
setlocal enabledelayedexpansion
for %%f in (test_code\*.krt) do (
  echo "fullname: %%f"
  cargo run %%f || exit /b 1
)
