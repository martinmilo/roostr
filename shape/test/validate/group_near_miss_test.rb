class GroupNearMissTest < AcceptanceTest
  def setup = workspace("atelier") # not under Git; the areas in entrypoints.tsv are Bookings, Workshops and Notifications

  ["Gift cards", "gift-cards", "GIFT_CARDS", "Giftcards"].each do |group|
    test "a group written #{group} while the scan writes the area GiftCards is a finding on the record's key group that names the area as the scan writes it" do
      write_file ".roostr/entrypoints.tsv", fixture("entrypoints/gift-cards-area.tsv") # Atelier's rows, and http POST /gift_cards in the area GiftCards
      write_file ".roostr/processes/sell-gift-card.yml", "name: Sell a gift card\ngroup: #{group}\ndescription: Take payment for a gift card and mail its code to the buyer.\n"

      validate

      assert_exit 1
      assert_stdout ".roostr/processes/sell-gift-card.yml: group: "
      assert_stdout "GiftCards"
    end
  end

  test "a group that resembles no area, Gift cards in Atelier, is allowed: no finding" do
    write_file ".roostr/processes/sell-gift-card.yml", "name: Sell a gift card\ngroup: Gift cards\ndescription: Take payment for a gift card and mail its code to the buyer.\n"

    validate

    assert_success
  end

  test "without entrypoints.tsv there are no areas to compare with: a group bookings is no finding, and the one finding is the missing entrypoints.tsv" do
    delete_file ".roostr/entrypoints.tsv"
    write_file ".roostr/processes/fix-double-booking.yml", "name: Fix a double booking\ngroup: bookings\ndescription: Move one of two attendees booked on the same place to another date.\n"

    validate

    assert_stdout_lines 1
    refute_stdout "fix-double-booking.yml"
  end

  test "validate --json gives the near miss as a problem of kind invalid on the key group, with a message naming the area" do
    write_file ".roostr/processes/fix-double-booking.yml", "name: Fix a double booking\ngroup: bookings\ndescription: Move one of two attendees booked on the same place to another date.\n"

    validate "--json"

    assert_exit 1
    assert_json "problems.*.file", [".roostr/processes/fix-double-booking.yml"]
    assert_json "problems.1.key", "group"
    assert_json "problems.1.message", /Bookings/
  end

  test "check reports the near miss under VALIDATE, exit 1" do
    write_file ".roostr/processes/fix-double-booking.yml", "name: Fix a double booking\ngroup: bookings\ndescription: Move one of two attendees booked on the same place to another date.\n"

    check

    assert_exit 1
    assert_stdout(/\AVALIDATE\n(.*\n)*    \.roostr\/processes\/fix-double-booking\.yml: group: .*Bookings/)
  end

  test "a record whose group is a near miss can still be read: the overview shows it under its own heading, and does not list it as needing checking" do
    write_file ".roostr/processes/fix-double-booking.yml", "name: Fix a double booking\ngroup: bookings\ndescription: Move one of two attendees booked on the same place to another date.\n"

    overview

    assert_success
    assert_stdout(/^  bookings\n    fix-double-booking /)
    refute_stdout "needs checking"
  end
end
