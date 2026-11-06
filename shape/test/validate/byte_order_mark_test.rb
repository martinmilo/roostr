class ByteOrderMarkTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes, 7 entrypoints, 5 described

  test "project.yml that starts with a byte order mark is read as without it: the overview prints, exit 0, starting with the name Atelier" do
    saved_with_a_byte_order_mark ".roostr/project.yml"

    overview

    assert_success
    assert_stdout(/\AAtelier +Rails 8/)
  end

  test "project.yml and a record that start with a byte order mark are no finding" do
    saved_with_a_byte_order_mark ".roostr/project.yml"
    saved_with_a_byte_order_mark ".roostr/processes/book-workshop.yml"

    validate

    assert_success
    assert_stdout "No problems found"
  end

  test "a record that starts with a byte order mark is readable: show prints it, exit 0" do
    saved_with_a_byte_order_mark ".roostr/processes/book-workshop.yml"

    show "book-workshop"

    assert_success
    assert_stdout(/\Abook-workshop +Book a place on a workshop/)
  end

  test "entrypoints.tsv that starts with a byte order mark is read as without it: its # covers: line counts, and the overview shows 7 entrypoints, 5 described" do
    saved_with_a_byte_order_mark ".roostr/entrypoints.tsv"

    overview

    assert_success
    assert_stdout "7 entrypoints, 5 described"
    assert_stdout "not scanned: schedule event cli"
  end
end
