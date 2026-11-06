class OverviewJsonNeedsAttentionTest < AcceptanceTest
  test "outside a Git repository, needs_attention is null: roostr cannot tell, which is not the same as nothing to attend to" do
    workspace "atelier"

    overview "--json"

    assert_success
    assert_json "needs_attention", nil
  end

  test "after a commit to booking.rb, needs_attention lists book-workshop and cancel-booking, in id order, each with that file" do
    atelier_under_git
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-06"

    overview "--json"

    assert_json "needs_attention", [{ "process" => "book-workshop", "files" => ["app/models/booking.rb"] },
                                    { "process" => "cancel-booking", "files" => ["app/models/booking.rb"] }]
  end

  test "in a Git repository where no process needs attention, needs_attention is an empty list" do
    atelier_under_git

    overview "--json"

    assert_json "needs_attention", []
  end

  test "in a Git repository with no commits yet, needs_attention is an empty list: Git was read, and a record never committed is skipped" do
    workspace "atelier"
    git "init", "--quiet"

    overview "--json"

    assert_json "needs_attention", []
  end

  test "when Git refuses the repository, roostr --json exits 0 with needs_attention null and everything else complete: the whole object is the overview of Atelier, as laid out" do
    atelier_under_git
    git_refuses_the_repository

    overview "--json"

    assert_success
    assert_json_is expected("overview_atelier.json")
  end

  test "in a shallow clone needs_attention is null: with the history cut short, roostr cannot tell which file changed after which record" do
    atelier_under_git
    edit_file "app/models/booking.rb"
    commit "Hold the place while the payment is pending", on: "2026-10-06" # with full history, book-workshop and cancel-booking would need attention
    shallow_clone

    overview "--json"

    assert_success
    assert_json "needs_attention", nil
  end
end
