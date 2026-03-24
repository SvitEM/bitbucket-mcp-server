# API Reference

Complete reference for all MCP tools provided by the Bitbucket MCP Server.

## Tool Categories

- [Projects](#projects) - 2 tools
- [Repositories](#repositories) - 2 tools
- [Pull Requests](#pull-requests) - 8 tools
- [Branches](#branches) - 3 tools
- [Commits](#commits) - 2 tools
- [Files](#files) - 2 tools

---

## Projects

### list_projects

List all projects with pagination support.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| start | number | No | Starting index for pagination (default: 0) |
| limit | number | No | Maximum number of results (default: 25) |

**Returns:** `PaginatedResponse<Project>`

**Example Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "list_projects",
    "arguments": {
      "start": 0,
      "limit": 10
    }
  }
}
```

**Example Response:**
```json
{
  "values": [
    {
      "key": "PROJ",
      "name": "My Project",
      "description": "Project description",
      "public": false,
      "type": "NORMAL"
    }
  ],
  "size": 1,
  "isLastPage": true
}
```

---

### get_project

Get details for a specific project.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key (e.g., "PROJ") |

**Returns:** `Project`

**Example Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_project",
    "arguments": {
      "projectKey": "PROJ"
    }
  }
}
```

**Example Response:**
```json
{
  "key": "PROJ",
  "name": "My Project",
  "description": "Project description",
  "public": false,
  "type": "NORMAL"
}
```

---

## Repositories

### list_repos

List repositories in a project.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| start | number | No | Starting index for pagination |
| limit | number | No | Maximum number of results |

**Returns:** `PaginatedResponse<Repository>`

**Example Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "list_repos",
    "arguments": {
      "projectKey": "PROJ",
      "limit": 10
    }
  }
}
```

---

### get_repo

Get details for a specific repository.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |

**Returns:** `Repository`

**Example Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_repo",
    "arguments": {
      "projectKey": "PROJ",
      "repositorySlug": "my-repo"
    }
  }
}
```

---

## Pull Requests

### list_prs

List pull requests in a repository.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| state | string | No | Filter by state: "OPEN", "MERGED", "DECLINED", "ALL" |
| start | number | No | Starting index for pagination |
| limit | number | No | Maximum number of results |

**Returns:** `PaginatedResponse<PullRequest>`

---

### get_pr

Get details for a specific pull request.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| pullRequestId | number | Yes | Pull request ID |

**Returns:** `PullRequest`

---

### create_pr

Create a new pull request.

**Permission Required:** WRITE

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| title | string | Yes | PR title |
| description | string | Yes | PR description |
| fromRef | string | Yes | Source branch (e.g., "refs/heads/feature") |
| toRef | string | Yes | Target branch (e.g., "refs/heads/main") |
| reviewers | array | No | Array of reviewer usernames |

**Returns:** `PullRequest`

**Example Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "create_pr",
    "arguments": {
      "projectKey": "PROJ",
      "repositorySlug": "my-repo",
      "title": "Add new feature",
      "description": "This PR adds a new feature",
      "fromRef": "refs/heads/feature-branch",
      "toRef": "refs/heads/main",
      "reviewers": ["john.doe"]
    }
  }
}
```

---

### update_pr

Update an existing pull request.

**Permission Required:** WRITE

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| pullRequestId | number | Yes | Pull request ID |
| version | number | Yes | Current PR version (for optimistic locking) |
| title | string | No | New title |
| description | string | No | New description |

**Returns:** `PullRequest`

---

### merge_pr

Merge a pull request.

**Permission Required:** WRITE

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| pullRequestId | number | Yes | Pull request ID |
| version | number | Yes | Current PR version |

**Returns:** `PullRequest`

---

### decline_pr

Decline a pull request.

**Permission Required:** DELETE

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| pullRequestId | number | Yes | Pull request ID |
| version | number | Yes | Current PR version |

**Returns:** `PullRequest`

---

### pr_diff

Get the diff for a pull request.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| pullRequestId | number | Yes | Pull request ID |

**Returns:** `string` (plain text diff)

---

### pr_changes

Get file changes for a pull request.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| pullRequestId | number | Yes | Pull request ID |
| start | number | No | Starting index for pagination |
| limit | number | No | Maximum number of results |

**Returns:** `PaginatedResponse<Change>`

---

## Branches

### list_branches

List branches in a repository.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| start | number | No | Starting index for pagination |
| limit | number | No | Maximum number of results |

**Returns:** `PaginatedResponse<Branch>`

---

### create_branch

Create a new branch.

**Permission Required:** WRITE

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| name | string | Yes | Branch name |
| startPoint | string | Yes | Starting commit/branch (e.g., "main" or commit hash) |

**Returns:** `Branch`

**Example Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "create_branch",
    "arguments": {
      "projectKey": "PROJ",
      "repositorySlug": "my-repo",
      "name": "feature-branch",
      "startPoint": "main"
    }
  }
}
```

---

### compare_branches

Compare commits between two branches.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| from | string | Yes | Source branch/commit |
| to | string | Yes | Target branch/commit |
| start | number | No | Starting index for pagination |
| limit | number | No | Maximum number of results |

**Returns:** `PaginatedResponse<Commit>`

---

## Commits

### list_commits

List commits in a repository.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| start | number | No | Starting index for pagination |
| limit | number | No | Maximum number of results |

**Returns:** `PaginatedResponse<Commit>`

---

### get_commit

Get details for a specific commit.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| commitId | string | Yes | Commit hash |

**Returns:** `Commit`

---

## Files

### list_files

Browse files in a repository.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| path | string | No | Directory path (default: root) |
| at | string | No | Branch/commit to browse (default: default branch) |

**Returns:** `PaginatedResponse<FileEntry>`

**Example Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "list_files",
    "arguments": {
      "projectKey": "PROJ",
      "repositorySlug": "my-repo",
      "path": "src",
      "at": "main"
    }
  }
}
```

---

### get_file_content

Get content of a file.

**Permission Required:** READ

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| projectKey | string | Yes | Project key |
| repositorySlug | string | Yes | Repository slug |
| path | string | Yes | File path |
| at | string | No | Branch/commit (default: default branch) |

**Returns:** `string` (file content)

**Example Request:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_file_content",
    "arguments": {
      "projectKey": "PROJ",
      "repositorySlug": "my-repo",
      "path": "src/main.rs",
      "at": "main"
    }
  }
}
```

---

## Common Types

### PaginatedResponse<T>

```typescript
{
  values: T[],
  size: number,
  isLastPage: boolean,
  nextPageStart?: number
}
```

### Project

```typescript
{
  key: string,
  name: string,
  description?: string,
  public: boolean,
  type: string
}
```

### Repository

```typescript
{
  slug: string,
  name: string,
  project: Project,
  public: boolean,
  links?: {
    clone: Array<{
      href: string,
      name: string
    }>
  }
}
```

### PullRequest

```typescript
{
  id: number,
  version: number,
  title: string,
  description?: string,
  state: "OPEN" | "MERGED" | "DECLINED",
  fromRef: Reference,
  toRef: Reference,
  author: Participant,
  reviewers: Participant[]
}
```

### Branch

```typescript
{
  id: string,
  displayId: string,
  type: string,
  latestCommit: string
}
```

### Commit

```typescript
{
  id: string,
  displayId: string,
  author: Author,
  authorTimestamp: number,
  message: string
}
```

### FileEntry

```typescript
{
  path: {
    components: string[],
    toString: string
  },
  type: "FILE" | "DIRECTORY",
  size?: number
}
```

---

## Error Handling

All tools return errors in the standard MCP error format:

```json
{
  "error": {
    "code": -32603,
    "message": "Error description",
    "data": null
  }
}
```

Common error scenarios:
- **Permission denied**: Operation blocked by ALLOW_READ/WRITE/DELETE settings
- **Authentication failed**: Invalid credentials or expired token
- **Not found**: Resource doesn't exist (404)
- **Invalid parameters**: Missing or malformed parameters
- **API error**: Bitbucket Server returned an error
