# Every test gets its own empty folder; commands run inside it.
#
#   workspace "shop"                 start from a copy of test/fixtures/workspaces/shop
#   workspace "shop", git: true      ...committed as a Git repository
#   workspace "shop", into: "apps/shop"   ...copied into that subfolder instead
#   commit "Redeem GIFT-2041"        stage everything and commit it
#   commit "...", on: "2026-10-05"   ...with that commit date; from: "second-checkout" in that worktree
#   branch "x" · switch_to "main" · merge "x", on: "2026-10-07"    a side branch, merged back
#   stage_everything_but ".roostr/x.yml"     git add every change but these paths: they stay unstaged
#                                    or untracked
#   stage_a_change_then_undo_it "app/x.rb"   the change is staged; the file is then as at HEAD again
#   folder_named "shop"              the test's folder is called shop; comes first in a test
#   write_file "notes.txt", "..."    add or replace a file
#   write_executable "bin/x", "..."  ...that can be run
#   named_pipe "p.yml"               a named pipe (FIFO) there: opening it to read waits for
#                                    a writer, which never comes
#   symbolic_link "z.yml", to: "/dev/zero"   a symbolic link there, to that path
#   edit_file "notes.txt"            change a file that is already there, without saying how
#   saved_with_a_byte_order_mark "notes.txt"   the file now starts with the bytes EF BB BF
#   saved_with_windows_line_endings "bin/x"    every line of the file now ends in CR LF
#   start_repository                 make the folder, as it is now, a Git repository with one commit
#   shallow_clone                    replace the repository by a clone of it of depth 1, every branch
#   shallow_clone depth: 2           ...of depth 2: the newest two commits of each branch
#   folder_outside_the_workspace     a new empty folder elsewhere, removed after the test: for a
#                                    fake program to put on the PATH
#   linked_worktree "second-checkout"   git worktree add: a second checkout of the repository in
#                                    that folder, on a new branch of the same name
#   remove_the_last_commit_for_good  git reset --hard HEAD~1, then the reflog expired and git gc:
#                                    the commit no longer exists in the repository
#
# Every folder the kit makes is taken by its real path: on macOS the temporary folder is
# reached through a symbolic link (/var is /private/var), and Git gives the real path.
#
#   assert_file "out/report.txt"                       it exists
#   assert_file "out/report.txt", includes: "3 rows"
#   assert_file "out/report.txt", is: expected("report.txt")
#   refute_file "out/report.txt"
#   assert_nothing_written           the last command left every file as it was
#   assert_nothing_written git_folder: true   ...the files in .git too
#   assert_written_only "out/a.txt"  ...created or changed this file and no other
#   assert_written_only_under ".git/roostr/"   ...wrote something, and created, changed or deleted
#                                    files in this folder and nowhere else, .git included
#   assert_unchanged "data.json"     the last command left this file as it was
require "open3"
require "tmpdir"
require "fileutils"
require "digest"

module Workspace
  GIT_ENV = {
    "GIT_AUTHOR_NAME" => "Acceptance Tests", "GIT_AUTHOR_EMAIL" => "tests@example.com",
    "GIT_COMMITTER_NAME" => "Acceptance Tests", "GIT_COMMITTER_EMAIL" => "tests@example.com",
    "GIT_AUTHOR_DATE" => "2026-01-15T09:00:00+01:00", "GIT_COMMITTER_DATE" => "2026-01-15T09:00:00+01:00",
    "GIT_CONFIG_GLOBAL" => File::NULL, "GIT_CONFIG_NOSYSTEM" => "1"
  }.freeze

  def dir
    @dir ||= File.realpath(Dir.mktmpdir("workspace-"))
  end

  # For behaviour that depends on what the folder is called.
  def folder_named(name)
    raise ArgumentError, "folder_named comes before anything else uses the test's folder" if @dir
    @parent = File.realpath(Dir.mktmpdir("workspace-"))
    note_situation "The folder the command runs in is called #{name}."
    @dir = File.join(@parent, name).tap { |path| Dir.mkdir(path) }
  end

  def workspace(name, git: false, into: nil)
    source = File.join(TEST_ROOT, "fixtures", "workspaces", name)
    raise ArgumentError, "No fixture workspace at #{source}" unless File.directory?(source)
    target = into ? File.join(dir, into) : dir
    FileUtils.mkdir_p(target)
    FileUtils.cp_r(File.join(source, "."), target)
    start_repository if git
  end

  # What a CI checkout fetches by default: the repository in the test's folder is replaced by
  # a clone of it with only the newest commit of each branch (depth 1), or the newest few
  # (depth: 2). The branch that was checked out is checked out again; every branch is also
  # there as origin/<name>.
  def shallow_clone(depth: 1)
    current = git("branch", "--show-current").strip
    origin = folder_outside_the_workspace("origin")
    FileUtils.cp_r(File.join(dir, "."), origin)
    FileUtils.rm_rf(Dir.children(dir).map { |child| File.join(dir, child) })
    git "clone", "--quiet", "--depth", depth.to_s, "--no-single-branch", "--branch", current, "file://#{origin}", "."
    remotes = git("branch", "--remotes", "--format=%(refname:short)").split.select { |name| name.include?("/") && !name.end_with?("/HEAD") }
    newest = depth == 1 ? "only the newest commit" : "only the newest #{depth} commits"
    note_situation "The folder is a shallow clone (depth #{depth}), as a CI checkout makes it: it has #{newest} of each branch, " \
                   "as #{remotes.join(" and ")}, and #{current} is checked out."
    raise ArgumentError, "shallow_clone depth: #{depth} cut nothing off: the history is no deeper than that" unless File.file?(File.join(dir, ".git/shallow"))
    edges = git("log", "--no-walk", "--format=%s", *read_file(".git/shallow").split).lines.map(&:chomp)
    note_situation "The clone holds these commits without their parents, at its edge: #{edges.join("; ")}."
  end

  # Not part of the workspace, so no command sees it there and assert_nothing_written ignores it.
  def folder_outside_the_workspace(name = "outside")
    File.realpath(Dir.mktmpdir("#{name}-")).tap { |path| (@other_folders ||= []) << path }
  end

  # A second checkout of the repository, as git worktree add makes it: a folder of the
  # workspace with a .git file, its own HEAD and index, and its own Git folder under
  # .git/worktrees/<name>. It is on a new branch named as the folder, at HEAD.
  def linked_worktree(path)
    git "worktree", "add", "--quiet", path
    note_situation "#{path} is a linked worktree of the repository (git worktree add #{path}): a second checkout, " \
                   "on the branch #{File.basename(path)}, with its own Git folder .git/worktrees/#{File.basename(path)}."
  end

  # The newest commit is undone and then removed from the repository for good: no branch, no
  # reflog entry and no object is left of it.
  def remove_the_last_commit_for_good
    subject = git("log", "-1", "--format=%s").chomp
    git "reset", "--quiet", "--hard", "HEAD~1"
    git "reflog", "expire", "--expire=now", "--all"
    git "gc", "--quiet", "--prune=now"
    note_situation "The commit \"#{subject}\" was undone (git reset --hard HEAD~1) and removed for good " \
                   "(git reflog expire --expire=now --all; git gc --prune=now): it no longer exists in the repository."
  end

  # Turns the folder as it is now into a Git repository with one commit, dated as GIT_ENV says.
  def start_repository
    git "init", "--quiet", "--initial-branch=main"
    commit "Initial state"
  end

  def write_file(path, content)
    FileUtils.mkdir_p(File.dirname(File.join(dir, path)))
    File.write(File.join(dir, path), content)
  end

  def write_executable(path, content)
    write_file path, content
    File.chmod(0o755, File.join(dir, path))
  end

  def named_pipe(path)
    FileUtils.mkdir_p(File.dirname(File.join(dir, path)))
    File.mkfifo(File.join(dir, path))
    note_situation "#{path} is a named pipe (FIFO) that nothing writes to: a program that opens it to read waits forever."
  end

  def symbolic_link(path, to:)
    FileUtils.mkdir_p(File.dirname(File.join(dir, path)))
    File.symlink(to, File.join(dir, path))
    note_situation "#{path} is a symbolic link to #{to}."
  end

  # Changes what is in a file without saying how: one more empty line at its end, which every
  # text format tolerates. For a change the program must notice in the content, use write_file.
  def edit_file(path)
    raise ArgumentError, "edit_file #{path}: there is no such file in the test's folder to edit; write_file creates one" unless File.file?(File.join(dir, path))
    write_file path, "#{read_file(path)}\n"
  end

  # As some Windows editors save a file: three bytes, EF BB BF, in front of its first line.
  def saved_with_a_byte_order_mark(path)
    write_file path, "﻿#{read_file(path)}"
    note_situation "#{path} starts with a UTF-8 byte order mark, the bytes EF BB BF, as some Windows editors save it."
  end

  # As a file saved on Windows: every line ends in CR LF, the first line too. Keeps the
  # executable bit.
  def saved_with_windows_line_endings(path)
    File.write(File.join(dir, path), read_file(path).gsub("\n", "\r\n"))
    note_situation "#{path} has Windows line endings: every line ends in CR LF, its first line too."
  end

  def delete_file(path) = FileUtils.rm(File.join(dir, path))
  def read_file(path)   = File.read(File.join(dir, path))

  def git(*args, env: {})
    output, status = Open3.capture2e(GIT_ENV.merge(env), "git", *args, chdir: dir)
    raise "git #{args.join(" ")} failed in the test's workspace:\n#{output}" unless status.success?
    output
  end

  # Without on: the commit has the fixed date in GIT_ENV. on: "2026-10-05" is noon UTC on that
  # day; a full timestamp, on: "2026-10-03T00:00:00+00:00", is taken as written. from:
  # "second-checkout" commits in that linked worktree.
  def commit(message = "Change", on: nil, from: nil)
    where = from ? ["-C", from] : []
    git(*where, "add", "--all")
    git(*where, "commit", "--quiet", "--allow-empty", "--message", message, env: commit_date(on))
  end

  # A side branch and back: branch "longer-reminders" starts one and moves to it,
  # switch_to "main" moves back, merge "longer-reminders" brings it in as a merge commit.
  def branch(name)    = git("switch", "--quiet", "--create", name)
  def switch_to(name) = git("switch", "--quiet", name)

  def merge(name, on: nil)
    git "merge", "--quiet", "--no-ff", "--message", "Merge #{name}", name, env: commit_date(on)
  end

  # What `git commit -am` and a hurried `git add` leave behind: every change staged, new files
  # included, except the paths given, which stay as they are in the working tree only.
  def stage_everything_but(*paths)
    git "add", "--all", "--", ".", *paths.map { |path| ":(exclude)#{path}" }
    note_situation "Every change is staged (git add --all) except #{paths.join(", ")}, which is not staged."
  end

  # The file's change is staged, and then the file is put back as it was: what is staged differs
  # from HEAD, the working tree does not.
  def stage_a_change_then_undo_it(path)
    before = read_file(path)
    write_file path, "#{before}# A change that was staged and then undone in the working tree.\n"
    git "add", "--", path
    write_file path, before
    note_situation "A change to #{path} is staged; the file in the working tree is as it was at HEAD again."
  end

  def assert_file(path, is: nil, includes: nil)
    assert File.file?(File.join(dir, path)), "Expected a file at #{path}.\n\n#{listing}#{transcript}"
    assert_same_text is, read_file(path), path, after: transcript if is
    assert read_file(path).include?(includes), "Expected #{path} to include #{includes.inspect}.\n\n  #{path}:\n#{read_file(path).gsub(/^/, "    ")}#{transcript}" if includes
  end

  def refute_file(path)
    refute File.exist?(File.join(dir, path)), "Expected no file at #{path}.\n\n#{listing}#{transcript}"
  end

  def assert_nothing_written(git_folder: false)
    changes = file_changes(git_folder: git_folder)
    assert changes.empty?, "Expected the command to leave every file as it was#{", in .git too" if git_folder}.\n\n#{changes.join("\n").gsub(/^/, "  ")}#{transcript}"
  end

  # Everything the last command created, changed or deleted, the files in .git included, is
  # in this folder, and there is something.
  def assert_written_only_under(folder)
    changes = file_changes(git_folder: true)
    outside = changes.reject { |change| change.split(/\s+/, 2).last.start_with?(folder) }
    shown = changes.empty? ? "  It wrote nothing." : changes.join("\n").gsub(/^/, "  ")
    assert !changes.empty? && outside.empty?,
           "Expected the command to write in #{folder} and nowhere else, .git included.\n\n#{shown}#{transcript}"
  end

  def assert_written_only(*paths)
    written = file_changes.reject { |change| change.start_with?("deleted") }.map { |change| change.split(/\s+/, 2).last }
    shown = file_changes.empty? ? "  It wrote nothing." : file_changes.join("\n").gsub(/^/, "  ")
    assert written.sort == paths.sort && written.size == file_changes.size,
           "Expected the command to write #{paths.join(", ")} and no other file.\n\n#{shown}#{transcript}"
  end

  def assert_unchanged(path)
    before = @remembered or raise Minitest::Assertion, "No command has been run in this test yet, so nothing can be unchanged."
    assert before.key?(path), "Expected the command to leave #{path} as it was, but there was no such file before the command.\n\n#{listing}#{transcript}"
    assert before[path] == snapshot[path], "Expected the command to leave #{path} as it was.#{transcript}"
  end

  def after_teardown
    super
    FileUtils.remove_entry(@parent || @dir) if @dir
    @other_folders&.each { |path| FileUtils.remove_entry(path) }
  end

  private

  def commit_date(on)
    return {} unless on
    moment = on.match?(/\A\d{4}-\d{2}-\d{2}\z/) ? "#{on}T12:00:00+00:00" : on
    unless moment.match?(/\A\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(Z|[+-]\d{2}:\d{2})\z/)
      raise ArgumentError, "commit on: takes a date or a timestamp with a zone, not #{on.inspect}"
    end
    { "GIT_AUTHOR_DATE" => moment, "GIT_COMMITTER_DATE" => moment }
  end

  # What Git holds in the test folder, for the transcript of a failure: the program reads it,
  # and nothing the command line shows would tell one history from another.
  def repository_state
    return [] unless File.directory?(File.join(dir, ".git"))
    log, status = Open3.capture2e(GIT_ENV, "git", "log", "--format=%x00%cI  %s", "--name-only", chdir: dir)
    return ["The folder is a Git repository with no commits yet."] unless status.success?
    commits = log.split("\0").reject(&:empty?).map do |entry|
      heading, *files = entry.lines.map(&:chomp).reject(&:empty?)
      files = ["(#{files.size} files)"] if files.size > 6
      ["  #{heading}", *files.map { |file| "      #{file}" }].join("\n")
    end
    pending = git("status", "--short", "--untracked-files=all").chomp
    pending = "#{pending.lines.first(10).join}… #{pending.lines.size - 10} more" if pending.lines.size > 20
    pending = pending.empty? ? "Nothing is uncommitted." : "Not committed, as git status --short shows it:\n#{pending.gsub(/^/, "  ")}"
    ["The folder is a Git repository. Its commits, newest first, with the commit date and the files each touched:\n#{commits.join("\n")}\n#{branches}#{pending}"]
  end

  # Dates do not show which commit follows which once a branch was merged; the graph does.
  def branches
    return "" if git("rev-list", "--merges", "HEAD").empty?
    "A branch was merged. The same commits as git log --graph draws them:\n#{git("log", "--graph", "--format=%s").chomp.gsub(/^/, "  ")}\n"
  end

  def remember_workspace
    @remembered = snapshot
    @remembered_with_git = snapshot(git_folder: true)
  end

  # What the last command did to the files: "created  path", "changed  path", "deleted  path".
  def file_changes(git_folder: false)
    before, after = git_folder ? [@remembered_with_git || {}, snapshot(git_folder: true)] : [@remembered || {}, snapshot]
    ((after.keys - before.keys).map { |path| "created  #{path}" } +
     (before.keys - after.keys).map { |path| "deleted  #{path}" } +
     (before.keys & after.keys).reject { |path| before[path] == after[path] }.map { |path| "changed  #{path}" }).sort
  end

  def transcript = @finished ? "\n\n#{@finished}" : ""

  def snapshot(git_folder: false)
    Dir.glob("**/*", File::FNM_DOTMATCH, base: dir)
       .reject { |path| (!git_folder && (path.start_with?(".git/") || path == ".git")) || !File.file?(File.join(dir, path)) }
       .to_h { |path| [path, Digest::SHA256.file(File.join(dir, path)).hexdigest] }
  end

  def listing
    files = snapshot.keys.sort
    files.empty? ? "  The workspace is empty." : "  Files in the workspace:\n#{files.join("\n").gsub(/^/, "    ")}"
  end
end
