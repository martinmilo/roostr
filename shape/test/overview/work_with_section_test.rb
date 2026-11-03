class WorkWithSectionTest < AcceptanceTest
  def setup = workspace("atelier") # operations start, test and deploy; deploy has a note

  test "an operation's note is printed on the lines under its command, never on the command's line, and wraps at 80 columns" do
    write_file ".roostr/project.yml", fixture("project/long-deploy-note.yml") # deploy has a note of 123 characters

    overview

    assert_stdout "  deploy    bin/kamal deploy\n" \
                  "            asks for the production vault password, which is in the shared\n" \
                  "            password manager under Atelier, and takes about four minutes\n"
    assert_lines_within 80
  end
end
