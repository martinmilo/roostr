class UnknownCommandTest < AcceptanceTest
  def setup = workspace("atelier")

  test "an unknown command is refused as a mistake in the command" do
    cli "frobnicate"

    assert_exit 2
    assert_nothing_written
  end

  test "roostr frobnicate --help is still an unknown command: exit 2, and stderr names it" do
    cli "frobnicate", "--help"

    assert_exit 2
    assert_stderr "frobnicate"
  end
end
