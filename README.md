# sophi

> sophi gives you the utility to make practical, lightweight, no-cost web applications with Google Apps Script.

It's opinionated and provides a base file structure that gives you the space you need for common features and room to grow.

### Key features
* **Project template:** Start your project right with the template
* **Secure OAuth:** Log in once, and you're running

## Getting started

### 1. Installation

Ensure you have the [Rust toolchain](https://rustup.rs/) installed, then run:

```bash
cargo install ---git "https://github.com/nick-luedde/sophi"
```

### 2. Get API credentials.

Follow the guide here to set up a GCP project and generate your client id and client secret.
<https://developers.google.com/identity/gsi/web/guides/get-google-api-clientid#get_your_google_api_client_id>

> Once you have your credentials, add them in a .env file in the install directory.

```text
CLIENT_ID=<your-gcp-client-id>
CLIENT_SECRET=<your-gcp-client-secret>
```

### 3. Start a new project.

You can start your project wherever you want.

```bash
cd my-workspace
sophi template "/project"
```

This sets up the file structure for creating a Google Spps Script project with sophi. More on that later.

### 4. Set up your `sophi.config.json file`

```json
{
  "driveUrl": "<google-drive-folder-url-where-script-project-and-other-resources-live>",
  "apps": [
    {
      "default": true,
      "name": "prodcution",
      "scriptId": "<your-script-id>",
      "devUrl": "<url-for-dev>",
      "deployment": {}
    }
  ],
  "script": [
    {
      "src": "./build/index.js",
      "to": "server/index"
    }
  ],
  "html": [
    {
      "src": "./build/index.html",
      "to": "client/index"
    }
  ]
}
```

This tells sophi which files to include, and which Google Apps Script project(s) to ship those files to.

#### Build for development

```bash
sophi build -a
```

Builds your app (for development) into ./build/index.js and ./build/index.html

#### Build for production

```bash
sophi build -a -p
```

#### Push to your Google Apps Script project

```bash
sophi push
```

Pushes your defined files (usually including you 'build' assets) to the Google Apps Script project

#### Build and push in one command: `ship`

```bash
sophi ship -a
```

Does all of the above. Add in the `-p` flag for production (Based on your ConfigEnj.js file)

## App directory structure

| Path | Description |
| :--- | :--- |
| `client/` | Root for all client side Vue.js, JavaScript, and CSS |
| `client/components` | Put all your Vue components here. Your sub-driectory structure can be as complex as you want. |
| `client/js` | Put your app js here. Think things like ViewModels and other utilities. |
| `client/main/app.js` | App setup (what happens wen your app first loads...) |
| `client/main/index.js` | Define your app routes |
| `client/main/index.html` | Your base app HTML. Leave the {{# ... }} tags as-is, but add anything else you want! |
| `server/` | Root for your server side files |
| `server/lib` | Put any server-side libraries here. Code that helps, but isn't strictly business logic. |
| `server/services` | Put all your business logic Google Apps Script here. I like to create a 'Services.js' for each logical 'domain' in the app. |
| `server/ConfigEnv.js` | Where you define 'development', 'test', and 'producion' envrionment configuration. |
| `server/server.js` | This is your entry point. If you want, this is where you can define what your server-side API is for your application. |
| `shared/` | If you have any JavaScript that you want to be able to run on both the client and the server, drop it here. |

You can take it from there. Use AppsSever.js if you want some clean server-side routing. Use SheetDataAccess.js if you want to use Google Sheets for a no-cost backend for your data.

On the client, get creative.  I like to use a public 'store' object that keeps application state, a layer of 'dispatchers' to handle sending requests to the Google Apps Script server, and 'view models' built with the Vue composition api.

See more: <https://developers.google.com/apps-script/guides/web>