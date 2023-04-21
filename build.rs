use std::error::Error;
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    // Emit the instructions
    EmitBuilder::builder()
        .all_build()
        .all_cargo()
        .git_branch()
        .git_commit_author_email()
        .git_commit_author_name()
        .git_commit_count()
        .git_commit_date()
        .git_commit_message()
        .git_commit_timestamp()
        .git_describe(true, true, None)
        .git_sha(false)
        .git_cmd(None)
        // .all_git()
        .all_rustc()
        .all_sysinfo()
        .emit()?;
    Ok(())
}
