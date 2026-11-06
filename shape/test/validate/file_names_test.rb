class FileNamesTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes; each test adds one record under a name of its own

  { "U+202E, a right-to-left override" => ["refund\u{202E}redro", "refund<U+202E>redro", "U+202E"],
    "ESC" => ["refund\e[31mred", "refund<U+001B>[31mred", "U+001B"],
    "a tab" => ["refund\tbooking", "refund<U+0009>booking", "U+0009"],
    "a newline" => ["refund\nbooking", "refund<U+000A>booking", "U+000A"],
    "a zero-width space" => ["refund\u{200B}-booking", "refund<U+200B>-booking", "U+200B"],
    "a soft hyphen, which a description may hold" => ["re\u{00AD}fund-booking", "re<U+00AD>fund-booking", "U+00AD"] }.each do |said, (name, shown, code)|
    test "a record whose file name holds #{said} is a finding about the file that names #{code} and says to rename it, with the file written #{shown}.yml" do
      write_file ".roostr/processes/#{name}.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\n"

      validate

      assert_exit 1
      assert_stdout_is ".roostr/processes/#{shown}.yml: the file's name holds #{code}, a character that is invisible or changes how text shows: rename the file\n"
    end
  end

  test "a file name holding ESC and a zero-width space gets one finding that names both, in the order they first appear" do
    write_file ".roostr/processes/refund\e[31m\u{200B}red.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\n"

    validate

    assert_exit 1
    assert_stdout_is ".roostr/processes/refund<U+001B>[31m<U+200B>red.yml: the file's name holds U+001B and U+200B, characters that are invisible or change how text shows: rename the file\n"
  end

  test "validate --json gives the file as it is, U+202E and all, key null, and a message that names U+202E" do
    write_file ".roostr/processes/refund\u{202E}redro.yml", "name: Refund a booking\ndescription: Refund a cancelled booking\n"

    validate "--json"

    assert_exit 1
    assert_json "problems.1.file", ".roostr/processes/refund\u{202E}redro.yml"
    assert_json "problems.1.key", nil
    assert_json "problems.1.message", /U\+202E/
  end
end
