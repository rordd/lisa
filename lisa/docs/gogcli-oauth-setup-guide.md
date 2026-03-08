# Google Cloud Console OAuth Client Setup Guide for gogcli

To use gogcli, you need to create an OAuth 2.0 client JSON file from Google Cloud Console. This guide covers the entire process step by step.

---

## Step 1: Access Google Cloud Console and Create a Project

1. Open [Google Cloud Console](https://console.cloud.google.com/) in your browser.
2. Sign in with your Google account.
3. Click the project selection dropdown in the top bar.
4. Click **"New Project"** in the upper right of the popup.
5. Enter a project name (e.g., `gogcli-project` or any name you prefer).
6. Leave the organization as default and click **"Create"**.
7. Once the project is created, select it from the notification at the top.

> **Note:** If you don't have a free trial account, you may see a prompt to sign up. You can create OAuth clients without enrolling in the free trial.

---

## Step 2: Enable Required APIs

You need to enable the Google service APIs that gogcli will use.

1. In the left menu, click **"APIs & Services"** -> **"Library"**.
2. Search for each API below and click **"Enable"**.

| Service | API Name |
|---------|----------|
| Gmail | Gmail API |
| Google Calendar | Google Calendar API |
| Google Drive | Google Drive API |
| Google Sheets | Google Sheets API |
| Google Docs | Google Docs API |
| Google Contacts | People API |
| Google Tasks | Tasks API |
| Google Slides | Google Slides API |

> **Tip:** You don't need to enable all APIs. Only enable the ones for services you'll use. For example, if you only need Gmail and Calendar, just enable those two.

---

## Step 3: Configure OAuth Consent Screen

You must set up the consent screen before creating an OAuth client.

1. In the left menu, click **"APIs & Services"** -> **"OAuth consent screen"**.
   - Or navigate to **"Google Auth Platform"** -> **"Branding"** (if the Google Cloud Console UI has been updated).
2. If you see **"Google Auth platform not configured yet"**, click **"Get Started"**.

### 3-1. App Information

| Field | Value |
|-------|-------|
| App name | `gogcli` (or any name you prefer) |
| User support email | Select your Gmail address |

Click **"Next"** after entering the information.

### 3-2. User Type (Audience)

- Personal Gmail users: Select **"External"**.
- Google Workspace organization users: You can select **"Internal"**.

Click **"Next"**.

### 3-3. Contact Information

- Enter your developer contact email address.

Click **"Next"**.

### 3-4. Complete

- Agree to the Google API Services User Data Policy and click **"Create"** or **"Save"**.

---

## Step 4: Register Test Users

If you selected "External", the app is created in **"Testing"** status. In this state, only registered test users can authenticate.

1. Navigate to **"Google Auth Platform"** -> **"Audience"**.
   - Or find the **"Test users"** tab under **"APIs & Services"** -> **"OAuth consent screen"**.
2. Click **"Add users"**.
3. Enter the Gmail address you'll use with gogcli.
4. Click **"Save"**.

> **Important:** Without registering as a test user, you'll get a `403: access_denied` error. Make sure to add your own email.

> **Note:** In testing mode, you can register up to 100 test users, and auth tokens expire after 7 days. Testing mode is sufficient for personal use.

---

## Step 5: Create OAuth Client ID and Download JSON

This is the key step. Here you'll create the `client_secret_xxx.json` file needed by gogcli.

1. In the left menu, click **"APIs & Services"** -> **"Credentials"**.
2. Click **"+ Create Credentials"** at the top.
3. Select **"OAuth client ID"** from the dropdown.
4. For **Application type**, select **"Desktop app"**.
   - **You must select "Desktop app".** Not "Web application".
5. Enter a name (e.g., `gogcli-desktop`).
6. Click **"Create"**.
7. The **"OAuth client created"** popup will appear.
   - It shows the **Client ID** and **Client Secret**.
8. Click the **"Download JSON"** button to download the JSON file.
   - The filename will be `client_secret_XXXXXXXXXXXX.json`.
9. Click **"OK"** to close the popup.

> **Security note:** The downloaded JSON file contains the client secret. Do not commit it to Git or share it with others.

---

## Step 6: Register OAuth Credentials in gogcli

Register the downloaded JSON file with gogcli.

```bash
# 1. Register the JSON file with gogcli
gog auth credentials ~/Downloads/client_secret_XXXXXXXXXXXX.json

# 2. Add Google account authentication
gog auth add you@gmail.com

# 3. When the browser opens, sign in to Google and grant permissions
#    If you see "This app isn't verified" warning:
#    Click "Advanced" -> "Go to gogcli (unsafe)"

# 4. Set default account (to skip --account each time)
export GOG_ACCOUNT=you@gmail.com

# 5. Verify it works
gog gmail labels list

# 6. List all calendar IDs (for USER.md setup)
gog calendar calendars -a you@gmail.com
```

---

## Additional Setup for Embedded Linux (BusyBox)

In embedded environments without a browser, use one of the following methods.

### Method A: Authenticate on Dev Host and Copy Config Files (Recommended)

1. Complete Step 6 above on your desktop/laptop.
2. Copy the entire config directory to the embedded board:

```bash
# Config directory locations:
# Linux: ~/.config/gogcli/
# macOS: ~/Library/Application Support/gogcli/

# Copy to embedded board
scp -r ~/.config/gogcli/ root@<board-IP>:~/.config/gogcli/
```

3. Set the keyring backend to file mode on the embedded board:

```bash
export GOG_KEYRING_BACKEND=file
export GOG_KEYRING_PASSWORD='your_password'
export GOG_ACCOUNT=you@gmail.com
```

### Method B: Manual/Headless Authentication Flow

```bash
# Run on the embedded board
gog auth add you@gmail.com --manual

# Copy the URL displayed in the terminal
# Open that URL in a desktop browser and complete authentication
# Copy the redirect URL and paste it back in the embedded board terminal
```

### Method C: Remote 2-Step Authentication Flow

```bash
# Step 1: Generate auth URL on the embedded board
gog auth add you@gmail.com --services user --remote --step 1

# Step 2: After authenticating in the browser, paste the redirect URL
gog auth add you@gmail.com --services user --remote --step 2 \
  --auth-url 'http://127.0.0.1:<port>/oauth2/callback?code=...&state=...'
```

---

## Common Issues and Solutions

### `403: access_denied` Error
- **Cause:** Attempting to authenticate with an email not registered as a test user.
- **Solution:** Go back to Step 4 and add your email as a test user.

### `This app isn't verified` Warning
- **This is normal.** As a personal project, there's no need for Google verification.
- Click **"Advanced"** -> **"Go to gogcli (unsafe)"**.

### Token Expiration (after 7 days)
- In testing mode, auth tokens expire every 7 days.
- Re-authenticate with `gog auth add you@gmail.com --force-consent`.

### Permission Error After Adding Scopes
- To use a new service (e.g., Sheets), re-authenticate with the `--force-consent` flag:
```bash
gog auth add you@gmail.com --services sheets --force-consent
```

### Keyring Error in Embedded Environments
- In environments without an OS keyring, use the file backend:
```bash
export GOG_KEYRING_BACKEND=file
export GOG_KEYRING_PASSWORD='any_password_here'
```

---

## Overall Flow Summary

```
Access Google Cloud Console
        |
   Create Project
        |
   Enable Required APIs (Gmail, Calendar, Drive, etc.)
        |
   Configure OAuth Consent Screen (External, enter app name)
        |
   Add Test Users (your email)
        |
   Credentials -> Create OAuth Client ID (Desktop app)
        |
   Download JSON File (client_secret_xxx.json)
        |
   gog auth credentials <json-file>
        |
   gog auth add <email>
        |
   Ready to use!
```
