class CheckCharactersTest < AcceptanceTest
  def setup = atelier_under_git # cancel-booking lists http DELETE /bookings/:id; nothing uncommitted, no scan script

  test "check writes a file name holding ESC as <U+001B> under VALIDATE, so that the report the terminal and the Stop hook show holds no raw control character" do
    write_file ".roostr/processes/refund\e[31mred.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\n"

    check

    assert_exit 1
    assert_stdout "    .roostr/processes/refund<U+001B>[31mred.yml: the file's name holds U+001B, a character that is invisible or changes how text shows: rename the file\n"
    refute_stdout "\e"
  end
end
