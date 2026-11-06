class NotRegularFilesTest < AcceptanceTest
  def setup = workspace("atelier") # 3 valid processes, entrypoints.tsv and project.yml; validate finds nothing

  test "a named pipe p.yml in .roostr/processes/ is not opened: validate reports it at once as not a regular file, exit 1" do
    named_pipe ".roostr/processes/p.yml"

    validate

    assert_exit 1
    assert_stdout_is ".roostr/processes/p.yml: not a regular file; roostr reads only regular files\n"
  end

  test "a record that is a symbolic link to /dev/null, a device, is not read: validate reports it as not a regular file, exit 1" do
    symbolic_link ".roostr/processes/empty.yml", to: "/dev/null" # a link to /dev/zero is the same case, which would run on forever

    validate

    assert_exit 1
    assert_stdout_is ".roostr/processes/empty.yml: not a regular file; roostr reads only regular files\n"
  end

  test "a named pipe in place of project.yml is not opened: validate reports it at once as not a regular file, exit 1" do
    delete_file ".roostr/project.yml"
    named_pipe ".roostr/project.yml"

    validate

    assert_exit 1
    assert_stdout ".roostr/project.yml: not a regular file; roostr reads only regular files\n"
  end

  test "a named pipe in place of entrypoints.tsv is not opened: validate reports it at once as not a regular file, exit 1" do
    delete_file ".roostr/entrypoints.tsv"
    named_pipe ".roostr/entrypoints.tsv"

    validate

    assert_exit 1
    assert_stdout ".roostr/entrypoints.tsv: not a regular file; roostr reads only regular files\n"
  end
end
