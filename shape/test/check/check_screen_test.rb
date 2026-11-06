class CheckScreenTest < AcceptanceTest
  def setup = atelier_halfway_through_the_waiting_list # the scan finds one entrypoint fewer, booking.rb is edited, two new entrypoints are listed by no record; nothing committed

  test "with problems of three kinds, check --base HEAD prints ENTRYPOINTS, PROCESSES and NOT DESCRIBED in that order, each a paragraph saying what to do and then its items, exactly as laid out" do
    check "--base", "HEAD"

    assert_exit 1
    assert_stdout_is expected("check_halfway_through_the_waiting_list.txt")
  end

  test "in that state, check leaves every file byte for byte as it was, entrypoints.tsv included, although it found it behind" do
    check "--base", "HEAD"

    assert_exit 1
    assert_nothing_written
  end

  test "with every change staged, check --staged prints the same screen as check --base HEAD, exactly as laid out" do
    git "add", "--all"

    check "--staged"

    assert_exit 1
    assert_stdout_is expected("check_halfway_through_the_waiting_list.txt")
  end
end
