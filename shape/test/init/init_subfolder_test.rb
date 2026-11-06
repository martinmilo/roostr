class InitSubfolderTest < AcceptanceTest
  test "init in apps/atelier, below the top of the repository where Git starts hooks, prints the pre-commit hook and the CI command with cd apps/atelier in front" do
    atelier_inside_a_larger_repository

    init from: "apps/atelier"

    assert_success
    assert_stdout "\n#!/bin/sh\ncd apps/atelier && roostr check --staged\n"
    assert_stdout "\ncd apps/atelier && roostr check --base origin/main\n"
  end

  test "init in apps/atelier puts cd \"$(git rev-parse --show-toplevel)/apps/atelier\" && in front of both Claude Code hooks, which run in any folder of the repository, with the Stop command in braces so that a failing cd does not block, and SessionStart given 10 seconds" do
    atelier_inside_a_larger_repository

    init from: "apps/atelier"

    assert_success
    assert_stdout '{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"cd \"$(git rev-parse --show-toplevel)/apps/atelier\" && roostr session start","timeout":10}]}],"Stop":[{"hooks":[{"type":"command","command":"cd \"$(git rev-parse --show-toplevel)/apps/atelier\" && { roostr check --session >&2 || { [ $? -eq 1 ] && exit 2; }; }"}]}]}}'
  end

  test "init in a folder whose name has a space, apps/my atelier, quotes it for sh: cd 'apps/my atelier' && roostr check --staged" do
    atelier_inside_a_larger_repository folder: "apps/my atelier"

    init from: "apps/my atelier"

    assert_success
    assert_stdout "\n#!/bin/sh\ncd 'apps/my atelier' && roostr check --staged\n"
  end

  test "init in apps/my atelier writes the folder as it is inside the double quotes of the Claude Code hooks: cd \"$(git rev-parse --show-toplevel)/apps/my atelier\" &&" do
    atelier_inside_a_larger_repository folder: "apps/my atelier"

    init from: "apps/my atelier"

    assert_success
    assert_stdout '"command":"cd \"$(git rev-parse --show-toplevel)/apps/my atelier\" && roostr session start","timeout":10}'
  end
end
