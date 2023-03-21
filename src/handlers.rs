use std::convert::Infallible;

use base64::{engine, Engine};
use chrono::{Days, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{File, FileIDAndOptionalGitHash, FileSummary, GitCommit, GitHistory, GitRef, PreviewDetail, PreviewDetailType};

pub(crate) async fn list_files() -> Result<impl warp::Reply, Infallible> {
    let example_files = [FileSummary {
        name: "README.md".to_string(),
        id: Uuid::nil(),
        edited_time: Utc::now(),
        created_time: Utc::now(),
    }, FileSummary {
        name: "main.rs".to_string(),
        id: Uuid::new_v4(),
        edited_time: Utc::now().checked_sub_days(Days::new(2)).unwrap(),
        created_time: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
    }];
    return Ok(warp::reply::json(&example_files));
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct NameOnly {
    name: String
}
pub(crate) async fn create_file(name: NameOnly) -> Result<impl warp::Reply, Infallible> {
    let example_file = FileSummary {
        name: name.name,
        id: Uuid::new_v4(),
        edited_time: Utc::now(),
        created_time: Utc::now(),
    };
    return Ok(warp::reply::json(&example_file));
}

pub(crate) async fn get_file(obj: FileIDAndOptionalGitHash) -> Result<impl warp::Reply, Infallible> {
    let example_file = File {
        name: "README.md".to_string(),
        id: obj.id,
        content: r####"## Prerequisites

You will need `nodejs` installed.

Node comes with its own package manager, `npm`. This will help us keep track of dependecies, which are annoying but unavoidable.

Some basics
- package.json is what tells npm what dependencies to install, how to build, and more. the package-lock.json makes sure everyone who builds has the same version of each package
  - You can add external packages by saying  `npm install <name> --save` (save will add it to package.json). Best to never edit package or package-lock manually
- eslint is a linter, which will look over javascript to make sure it follows conventions

## Getting Started


First, run the development server:

```bash
npm run dev
```
Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

You can start editing the page by modifying `pages/index.js`. The page auto-updates as you edit the file.

The button on the page will call an external api (see `api/hello.js`). This is how we will call the external API to do the git commands. You'll notice that we load the url by saying `process.env.<name>`. This is using an environment variable. This means varibles floating around in the shell when this app is run. You have tons of variables in your shell environment. You can look at them by running the `printenv` command.

To load the url into your environment, make a file called `.env` in the root directory. Then, put this line in

```
BORED_API_URL=https://www.boredapi.com/api/activity
```

now, when you run `npm run dev`, the variable will be loaded into your environment. Now when you look a the page, you can click the button to hit the bored api, which gives an activity to do when you are bored. To see the output, look in the console (left click on the browser page, hit inspect, and go to console in the dev tools window).

## Learn More

I would read some mozilla tutorials on html, css, and javascript. Also reading an intro to react would help

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features and API.
- [Learn Next.js](https://nextjs.org/learn) - an interactive Next.js tutorial.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js/) - your feedback and contributions are welcome!
"####.to_string(),
    };
    return Ok(warp::reply::json(&example_file));
}

pub(crate) async fn save_file(obj: File) -> Result<impl warp::Reply, Infallible> {
    let example_commit = GitCommit {
        hash: "aceaaec23664ae26d76ab66cedfb1206b9c972b1".to_string(),
        parent: None,
    };
    return Ok(warp::reply::json(&example_commit));
}

pub(crate) async fn preview_file(obj: FileIDAndOptionalGitHash) -> Result<impl warp::Reply, Infallible> {
    return Ok(warp::reply());
}

pub(crate) async fn get_preview(obj: FileIDAndOptionalGitHash) -> Result<impl warp::Reply, Infallible> {
    let result = PreviewDetail {
        name: "README.md".to_string(),
        id: obj.id,
        r#type: PreviewDetailType::HTML,
        data: engine::general_purpose::STANDARD_NO_PAD.encode(r###"
    <h2 id="prerequisites">Prerequisites</h2>
<p>You will need <code>nodejs</code> installed.</p>
<p>Node comes with its own package manager, <code>npm</code>. This will
help us keep track of dependecies, which are annoying but
unavoidable.</p>
<p>Some basics - package.json is what tells npm what dependencies to
install, how to build, and more. the package-lock.json makes sure
everyone who builds has the same version of each package - You can add
external packages by saying <code>npm install &lt;name&gt; --save</code>
(save will add it to package.json). Best to never edit package or
package-lock manually - eslint is a linter, which will look over
javascript to make sure it follows conventions</p>
<h2 id="getting-started">Getting Started</h2>
<p>First, run the development server:</p>
<div class="sourceCode" id="cb1"><pre
class="sourceCode bash"><code class="sourceCode bash"><span id="cb1-1"><a href="#cb1-1" aria-hidden="true" tabindex="-1"></a><span class="ex">npm</span> run dev</span></code></pre></div>
<p>Open <a href="http://localhost:3000">http://localhost:3000</a> with
        your browser to see the result.</p>
        <p>You can start editing the page by modifying
        <code>pages/index.js</code>. The page auto-updates as you edit the
    file.</p>
        <p>The button on the page will call an external api (see
        <code>api/hello.js</code>). This is how we will call the external API to
    do the git commands. You’ll notice that we load the url by saying
        <code>process.env.&lt;name&gt;</code>. This is using an environment
    variable. This means varibles floating around in the shell when this app
    is run. You have tons of variables in your shell environment. You can
    look at them by running the <code>printenv</code> command.</p>
        <p>To load the url into your environment, make a file called
        <code>.env</code> in the root directory. Then, put this line in</p>
        <pre><code>BORED_API_URL=https://www.boredapi.com/api/activity</code></pre>
    <p>now, when you run <code>npm run dev</code>, the variable will be
    loaded into your environment. Now when you look a the page, you can
    click the button to hit the bored api, which gives an activity to do
    when you are bored. To see the output, look in the console (left click
                                                                on the browser page, hit inspect, and go to console in the dev tools
                                                                window).</p>
        <h2 id="learn-more">Learn More</h2>
        <p>I would read some mozilla tutorials on html, css, and javascript.
        Also reading an intro to react would help</p>
        <p>To learn more about Next.js, take a look at the following
    resources:</p>
        <ul>
        <li><a href="https://nextjs.org/docs">Next.js Documentation</a> - learn
    about Next.js features and API.</li>
        <li><a href="https://nextjs.org/learn">Learn Next.js</a> - an
    interactive Next.js tutorial.</li>
        </ul>
        <p>You can check out <a href="https://github.com/vercel/next.js/">the
    Next.js GitHub repository</a> - your feedback and contributions are
    welcome!</p>
        "###)
    };
    return Ok(warp::reply::json(&result));
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct IdOnly {
    id: Uuid
}
pub(crate) async fn get_history(file_id: IdOnly) -> Result<impl warp::Reply, Infallible> {
    let example_git_history = GitHistory {
        commits: vec![
            GitCommit { hash: "aceaaec23664ae26d76ab66cedfb1206b9c972b1".to_string(), parent: None },
            GitCommit { hash: "7c570dce251232eecd2daa6bd81723ef0a1a7590".to_string(), parent: Some("aceaaec23664ae26d76ab66cedfb1206b9c972b1".to_string()) }
        ],
        refs: vec![
            GitRef { name: "main".to_string(), hash: "7c570dce251232eecd2daa6bd81723ef0a1a7590".to_string() }
        ],
    };
    return Ok(warp::reply::json(&example_git_history));
}