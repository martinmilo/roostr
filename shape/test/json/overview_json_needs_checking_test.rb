class OverviewJsonNeedsCheckingTest < AcceptanceTest
  def setup = workspace("atelier") # book-workshop, cancel-booking and remind-attendees, all readable

  test "a record that does not parse is left out of processes and is in needs_checking by its file, and the other records are still in processes" do
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"

    overview "--json"

    assert_success
    assert_json "processes.*.id", ["book-workshop", "remind-attendees"]
    assert_json "needs_checking", [{ "file" => ".roostr/processes/cancel-booking.yml" }]
  end

  test "needs_checking is in the order of the paths: .roostr/entrypoints.tsv, then the records by file name" do
    write_file ".roostr/processes/remind-attendees.yml", "name: [unclosed\n"
    write_file ".roostr/processes/cancel-booking.yml", "name: [unclosed\n"
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/unknown-kind.tsv") # the 7 rows of Atelier, then one of kind webhook

    overview "--json"

    assert_json "needs_checking.*.file", [".roostr/entrypoints.tsv", ".roostr/processes/cancel-booking.yml",
                                          ".roostr/processes/remind-attendees.yml"]
  end

  {
    "why written as one string" => "name: Refund a booking\ndescription: Refund a cancelled booking\nwhy: Cards are refunded, never vouchers.\n",
    "an unknown key, outcome: written for outcomes:" => "name: Refund a booking\ndescription: Refund a cancelled booking\noutcome:\n  refunded:\n    when: the booking was cancelled\n    then: the card is refunded\n",
    "no description" => "name: Refund a booking\n",
    "a checked date that does not exist, 2026-02-30" => "name: Refund a booking\ndescription: Refund a cancelled booking\nchecked: 2026-02-30\n",
    "an entrypoint of a kind outside the closed list" => "name: Refund a booking\ndescription: Refund a cancelled booking\nentrypoints:\n  - webhook stripe.charge.refunded\n"
  }.each do |problem, record|
    test "a record with #{problem} has a finding about its own content, so it is in needs_checking and not in processes" do
      write_file ".roostr/processes/refund-booking.yml", record

      overview "--json"

      assert_json "needs_checking", [{ "file" => ".roostr/processes/refund-booking.yml" }]
      assert_json "processes.*.id", ["book-workshop", "cancel-booking", "remind-attendees"]
    end
  end

  {
    "export-bookings" => "lists a file that does not exist",
    "refund-booking" => "lists an entrypoint that is not in entrypoints.tsv"
  }.each do |id, problem|
    test "a record whose only finding is about another file, here #{id}, which #{problem}, is in processes and not in needs_checking" do
      write_file ".roostr/processes/#{id}.yml", fixture("processes/#{id}.yml")

      overview "--json"

      assert_json "processes.3.id", id
      assert_json "needs_checking", []
    end
  end
end
