class SessionMapGroupsTest < AcceptanceTest
  def setup = atelier_under_git # groups Bookings (book-workshop, cancel-booking) and Notifications (remind-attendees)

  test "a record whose group is other and one with no group are one group in the map: a single other:, last, after stock, a group whose name sorts after other" do
    records_added "order-clay", "import-workshops", "count-stock" # group other, no group, group stock

    session_start

    assert_map(/\nNotifications:\n- remind-attendees: .*\nstock:\n- count-stock: .*\nother:\n- import-workshops: .*\n- order-clay: .*\z/)
  end

  test "in the map shrunk to one line of ids per group, other is one line too, last: the process with no group and the one in group other together" do
    processes_written 200, groups: 6
    records_added "order-clay", "import-workshops", "count-stock" # group other, no group, group stock

    session_start

    assert_map "\nNotifications: remind-attendees\nstock: count-stock\nother: import-workshops, order-clay\n\nLeft out to keep this map within 8,000 bytes"
  end

  test "a group, a description and a first file that are each empty once the characters that do not show are dropped count as absent: the process is under other, as - refund-booking [app/models/booking.rb]" do
    write_file ".roostr/processes/refund-booking.yml", fixture("processes/refund-booking-with-values-of-hidden-characters-only.yml")

    session_start

    assert_map(/\nother:\n- refund-booking \[app\/models\/booking\.rb\]\z/)
  end

  test "groups are told apart by their name as the map prints it: Book, a zero-width space, ings and Bookings are one group, Bookings" do
    write_file ".roostr/processes/move-booking.yml", fixture("processes/move-booking-in-a-group-with-a-zero-width-space.yml")

    session_start

    assert_map(/\nBookings:\n- book-workshop: .*\n- cancel-booking: .*\n- move-booking: .*\nNotifications:\n/)
  end
end
