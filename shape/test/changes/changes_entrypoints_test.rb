class ChangesEntrypointsTest < AcceptanceTest
  def setup = atelier_under_git # committed once on 2026-01-15 with 7 entrypoints; today is 2026-10-10

  test "the entrypoints.tsv in the folder, committed or not, is compared with the one committed before the period: new entrypoints follow a plus sign, missing ones a minus sign" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/booking-page-dropped.tsv") # adds POST /waiting_list and WaitingListJob, drops GET /bookings/:id; not committed

    changes

    assert_stdout "ENTRYPOINTS\n  + http POST /waiting_list\n  + job WaitingListJob\n  - http GET /bookings/:id\n"
    assert_stdout "entrypoints: 2 added, 1 removed"
  end

  test "an entrypoint that was already in the entrypoints.tsv committed before the period started is not listed as added" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/waiting-list-added.tsv") # adds POST /waiting_list and WaitingListJob
    commit "Add the waiting list", on: "2026-10-02"

    changes "--since", "2026-10-03"

    assert_stdout "entrypoints: 0 added, 0 removed"
    refute_stdout "ENTRYPOINTS"
  end

  test "an entrypoint whose file changed is neither added nor removed" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/booking-moved.tsv") # the same 7; POST /bookings is now in checkout_controller.rb

    changes

    assert_stdout "entrypoints: 0 added, 0 removed"
    refute_stdout "POST /bookings"
  end
end
