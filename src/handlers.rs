use std::convert::Infallible;

use base64::{engine, Engine};
use chrono::{Days, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::hyper::StatusCode;

use crate::api::{CompilationOutput, CompilationState, File, FileIDAndOptionalGitHash, FileSummary, GitCommit, GitHistory, GitRef, PreviewDetail, PreviewDetailType};

// Fetches each file/repository info for each file on disk, and return it via the api
// -- Currently, this returns and responds with fake data
pub(crate) async fn list_files() -> Result<impl warp::Reply, Infallible> {
    let example_files = if rand::random() {
        vec![]
    } else {
        vec![FileSummary {
            name: "README.md".to_string(),
            id: Uuid::nil(),
            edited_time: Utc::now(),
            created_time: Utc::now(),
        }, FileSummary {
            name: "main.rs".to_string(),
            id: Uuid::new_v4(),
            edited_time: Utc::now().checked_sub_days(Days::new(2)).unwrap(),
            created_time: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
        }]
    };
    return Ok(warp::reply::json(&example_files));
}

// The NameOnly strut allows a string to be serialized into JSON data which warp
// formats and uses easier
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct NameOnly {
    name: String
}
// Create file creates a new file object, and returns the data to the client via the api
pub(crate) async fn create_file(name: NameOnly) -> Result<impl warp::Reply, Infallible> {
    // Create a new file summary strut
    let example_file = FileSummary {
        name: name.name,
        id: Uuid::new_v4(),
        edited_time: Utc::now(),
        created_time: Utc::now(),
    };
    return Ok(warp::reply::json(&example_file));
}

// Get file will take a file ID/git hash and, if the file is found successfully, send the file
// to the client while returning a result of success or failure
// -- Currently, this function randomly responds with either a "file not found" message or
// -- sends the README file to the client
pub(crate) async fn get_file(obj: FileIDAndOptionalGitHash) -> Result<Box<dyn warp::Reply>, Infallible> {
    return if rand::random() {
        Ok(Box::new(StatusCode::NOT_FOUND))
    } else {
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
        Ok(Box::new(warp::reply::json(&example_file)))
    }
}

// Save file will take a file object/instance, and will respond with a GitCommit object to the client,
// while returning the success/failure result
pub(crate) async fn save_file(obj: File) -> Result<Box<dyn warp::Reply>, Infallible> {
    return if rand::random() {
        let example_commit = GitCommit {
            hash: "aceaaec23664ae26d76ab66cedfb1206b9c972b1".to_string(),
            parent: None,
        };
        Ok(Box::new(warp::reply::json(&example_commit)))
    } else if rand::random() {
        let example_commit = GitCommit {
            hash: "7c570dce251232eecd2daa6bd81723ef0a1a7590".to_string(),
            parent: Some("aceaaec23664ae26d76ab66cedfb1206b9c972b1".to_string()),
        };
        Ok(Box::new(warp::reply::json(&example_commit)))
    } else {
        Ok(Box::new(StatusCode::FORBIDDEN))
    }
}

// Preview file will take a file ID with an optional git hash, and respond with a success or failure message to the
// client, while returning a result
// -- Currently, this request will randomly respond with SUCCESS, FAILURE, or NOT FOUND
pub(crate) async fn preview_file(obj: FileIDAndOptionalGitHash) -> Result<Box<dyn warp::Reply>, Infallible> {
    return if rand::random() {
        Ok(Box::new(warp::reply::json(&CompilationOutput {
            state: CompilationState::SUCCESS,
            log: "".to_string(),
        })))
    } else if rand::random() {
        Ok(Box::new(warp::reply::json(&CompilationOutput {
            state: CompilationState::FAILURE,
            log: "".to_string(),
        })))
    } else {
        Ok(Box::new(StatusCode::NOT_FOUND))
    };
}

// Get preview will take a file ID with an optional git hash, and respond with a preview file object to the client,
// while also returning a result
// -- Currently, this request responds with a PreviewDetail object of README or a NOT_FOUND message at random
pub(crate) async fn get_preview(obj: FileIDAndOptionalGitHash) -> Result<Box<dyn warp::Reply>, Infallible> {
    return if rand::random() {
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
        Ok(Box::new(warp::reply::json(&result)))
    // } else if rand::random() {
    //     let pdf = 
    //     let result = PreviewDetail {
    //         name: "README.md".to_string(),
    //         id: obj.id,
    //         r#type: PreviewDetailType::PDF,
    //         data: pdf
    //     };
    //     Ok(Box::new(warp::reply::json(&result)))
    } else {
        Ok(Box::new(StatusCode::NOT_FOUND))
    };
}

// The IdOnly strut allows a string to be serialized into JSON data which warp
// formats and uses easier
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct IdOnly {
    id: Uuid
}
// Get History takes a file ID, and responds with a list of GitCommit objects and a list of
// GitRef objects, representing the file's git lineage, while returning the result
// -- Currently, this request responds with either fake data, empty history, or an NOT FOUND code
// -- at random
pub(crate) async fn get_history(file_id: IdOnly) -> Result<Box<dyn warp::Reply>, Infallible> {
    if rand::random() {
        let example_git_history = GitHistory {
            commits: vec![
                GitCommit { hash: "aceaaec23664ae26d76ab66cedfb1206b9c972b1".to_string(), parent: None },
                GitCommit { hash: "7c570dce251232eecd2daa6bd81723ef0a1a7590".to_string(), parent: Some("aceaaec23664ae26d76ab66cedfb1206b9c972b1".to_string()) }
            ],
            refs: vec![
                GitRef { name: "main".to_string(), hash: "7c570dce251232eecd2daa6bd81723ef0a1a7590".to_string() }
            ],
        };
        return Ok(Box::new(warp::reply::json(&example_git_history)))
    } else if rand::random() {
        let example_git_history = GitHistory {
            commits: vec![],
            refs: vec![],
        };
        return Ok(Box::new(warp::reply::json(&example_git_history)))
    } else {
        return Ok(Box::new(StatusCode::NOT_FOUND));
    }
}