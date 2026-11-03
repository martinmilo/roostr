class EntrypointCountsTest < AcceptanceTest
  def setup = workspace("atelier") # entrypoints.tsv covers http, job and mail; the two mail entrypoints have no area

  test "other counts only the kinds that have entrypoints without an area" do
    overview

    assert_stdout(/^  other\s+mail 2$/)
    refute_stdout "http 0"
  end

  test "a kind the scan covers but found nowhere reads none found, never 0" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/covers-schedule-too.tsv") # same rows, header also covers schedule

    overview

    assert_stdout "none found: schedule"
    refute_stdout "schedule 0"
  end

  test "a kind the scan does not cover reads not scanned, never 0" do
    overview

    assert_stdout "not scanned: schedule event cli"
    refute_stdout "schedule 0"
  end

  test "other is left out when every entrypoint has an area" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/all-areas.tsv") # the two mails now belong to Notifications

    overview

    assert_stdout "7 entrypoints, 5 described"
    refute_stdout(/^  other/)
  end

  test "the coverage line is left out when every kind is covered and found" do
    write_file ".roostr/entrypoints.tsv", fixture("entrypoints/all-kinds.tsv") # covers all six kinds, one schedule, event and cli entrypoint added

    overview

    assert_stdout "10 entrypoints, 5 described"
    refute_stdout "none found"
    refute_stdout "not scanned"
  end

  test "with no entrypoints.tsv the FUNCTIONALITY heading reads not scanned yet and no count is given" do
    delete_file ".roostr/entrypoints.tsv"

    overview

    assert_success
    assert_stdout(/FUNCTIONALITY\s+not scanned yet/)
    refute_stdout "described"
  end
end
