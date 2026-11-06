class CheckMergeInProgressTest < AcceptanceTest
  def setup = atelier_under_git # booking.rb is listed by book-workshop and cancel-booking; workshops_controller.rb by no record

  test "during a merge of main, Git stopped before committing, what Git merged from main is not the work: check --base HEAD prints No problems found" do
    merging_main on_main: ["app/models/booking.rb"], on_the_branch: ["app/controllers/workshops_controller.rb"]

    check "--base", "HEAD"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "during the same merge, in a session that has made no commit yet, check --session answers as check --base HEAD: No problems found" do
    merging_main on_main: ["app/models/booking.rb"], on_the_branch: ["app/controllers/workshops_controller.rb"]
    session_start

    check_session

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "during the same merge, check --staged prints No problems found: the changes from main that Git staged are not the work either" do
    merging_main on_main: ["app/models/booking.rb"], on_the_branch: ["app/controllers/workshops_controller.rb"]

    check "--staged"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a merge stopped on a conflict in booking.rb, resolved by taking main's side, is not work: No problems found" do
    merging_main on_main: ["app/models/booking.rb"], on_the_branch: ["app/models/booking.rb"]
    take_mains_side "app/models/booking.rb"

    check "--base", "HEAD"

    assert_success
    assert_stdout_is "No problems found\n"
  end

  test "a conflict in booking.rb resolved by hand, to a version neither side had, is work: PROCESSES lists booking.rb under book-workshop and cancel-booking, exit 1" do
    merging_main on_main: ["app/models/booking.rb"], on_the_branch: ["app/models/booking.rb"]
    resolve_by_hand "app/models/booking.rb"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "\n    book-workshop\n      app/models/booking.rb\n    cancel-booking\n      app/models/booking.rb\n"
  end

  test "a conflict resolved by hand, with both records edited in the merge as well, leaves nothing to report" do
    merging_main on_main: ["app/models/booking.rb"], on_the_branch: ["app/models/booking.rb"]
    resolve_by_hand "app/models/booking.rb"
    edit_file ".roostr/processes/book-workshop.yml"
    edit_file ".roostr/processes/cancel-booking.yml"

    check "--base", "HEAD"

    assert_success
  end

  test "a record the merge takes from main as main left it is not updated by the merge: a conflict resolved by hand still lists cancel-booking, whose record main edited" do
    merging_main on_main: ["app/models/booking.rb", ".roostr/processes/cancel-booking.yml"], on_the_branch: ["app/models/booking.rb"]
    resolve_by_hand "app/models/booking.rb"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "\n    cancel-booking\n      app/models/booking.rb\n"
  end

  test "during a merge of main, check --base main answers as it will once the merge is committed: the branch's own change to booking.rb is listed under cancel-booking, whose record only main edited" do
    merging_main on_main: [".roostr/processes/cancel-booking.yml"], on_the_branch: ["app/models/booking.rb"]

    check "--base", "main"

    assert_exit 1
    assert_stdout "\n    cancel-booking\n      app/models/booking.rb\n"
  end

  test "concluding a merge whose conflict took main's side, git commit with the pre-commit hook roostr check --staged makes the merge commit" do
    merging_main on_main: ["app/models/booking.rb"], on_the_branch: ["app/models/booking.rb"]
    take_mains_side "app/models/booking.rb"
    pre_commit_hook "roostr check --staged"

    git_commit "--no-edit"

    assert_success
    assert_commit_made
  end

  test "a cherry-pick in progress is not a merge: the picked change to booking.rb is the work, and check --base HEAD lists it" do
    cherry_picking_a_change_to "app/models/booking.rb"

    check "--base", "HEAD"

    assert_exit 1
    assert_stdout "\n    cancel-booking\n      app/models/booking.rb\n"
  end
end
