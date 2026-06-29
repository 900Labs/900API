<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { Users, Plus, Trash2, UserPlus, Crown, Shield, Edit3, Eye, Activity, Share2 } from '@lucide/svelte'

  type TeamRole = 'owner' | 'admin' | 'editor' | 'viewer'

  type TeamMember = {
    id: string
    name: string
    email: string
    role: TeamRole
    avatar_color: string
    last_active: string
  }

  type Workspace = {
    id: string
    name: string
    description: string | null
    owner_id: string
    members: TeamMember[]
    collection_ids: string[]
    environment_ids: string[]
    created_at: string
  }

  type ActivityEvent = {
    id: string
    workspace_id: string
    user_id: string
    user_name: string
    action: string
    resource_type: string
    resource_name: string
    timestamp: string
  }

  let workspaces = $state<Workspace[]>([])
  let selectedWorkspaceId = $state<string | null>(null)
  let activities = $state<ActivityEvent[]>([])
  let loading = $state(false)
  let error = $state<string | null>(null)
  let success = $state<string | null>(null)
  let showCreateForm = $state(false)
  let showAddMember = $state(false)

  let newWorkspaceName = $state('')
  let newWorkspaceDesc = $state('')
  let newMemberName = $state('')
  let newMemberEmail = $state('')
  let newMemberRole = $state<TeamRole>('editor')

  const roleIcons: Record<TeamRole, typeof Crown> = {
    owner: Crown,
    admin: Shield,
    editor: Edit3,
    viewer: Eye,
  }

  const roleColors: Record<TeamRole, string> = {
    owner: 'text-warning',
    admin: 'text-accent',
    editor: 'text-success',
    viewer: 'text-text-muted',
  }

  async function loadWorkspaces() {
    loading = true
    error = null
    try {
      workspaces = await invoke<Workspace[]>('team_list_workspaces')
      if (workspaces.length > 0 && !selectedWorkspaceId) {
        selectedWorkspaceId = workspaces[0].id
        await loadActivity()
      }
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function loadActivity() {
    if (!selectedWorkspaceId) return
    try {
      activities = await invoke<ActivityEvent[]>('team_get_activity', {
        workspaceId: selectedWorkspaceId,
        limit: 20,
      })
    } catch {
      // ignore
    }
  }

  async function createWorkspace() {
    error = null
    success = null
    try {
      const owner: TeamMember = {
        id: crypto.randomUUID(),
        name: 'You',
        email: 'you@900api.dev',
        role: 'owner',
        avatar_color: '#6366f1',
        last_active: new Date().toISOString(),
      }
      const ws = await invoke<Workspace>('team_create_workspace', {
        name: newWorkspaceName,
        description: newWorkspaceDesc || null,
        owner,
      })
      newWorkspaceName = ''
      newWorkspaceDesc = ''
      showCreateForm = false
      await loadWorkspaces()
      selectedWorkspaceId = ws.id
      success = 'Workspace created'
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    }
  }

  async function deleteWorkspace(id: string) {
    try {
      await invoke('team_delete_workspace', { id })
      if (selectedWorkspaceId === id) {
        selectedWorkspaceId = null
      }
      await loadWorkspaces()
      success = 'Workspace deleted'
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    }
  }

  async function addMember() {
    if (!selectedWorkspaceId) return
    error = null
    try {
      const member: TeamMember = {
        id: crypto.randomUUID(),
        name: newMemberName,
        email: newMemberEmail,
        role: newMemberRole,
        avatar_color: `hsl(${Math.random() * 360}, 70%, 50%)`,
        last_active: new Date().toISOString(),
      }
      await invoke('team_add_member', {
        workspaceId: selectedWorkspaceId,
        member,
      })
      newMemberName = ''
      newMemberEmail = ''
      newMemberRole = 'editor'
      showAddMember = false
      await loadWorkspaces()
      success = 'Member added'
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    }
  }

  async function removeMember(memberId: string) {
    if (!selectedWorkspaceId) return
    try {
      await invoke('team_remove_member', {
        workspaceId: selectedWorkspaceId,
        memberId,
      })
      await loadWorkspaces()
      success = 'Member removed'
      setTimeout(() => (success = null), 3000)
    } catch (e) {
      error = String(e)
    }
  }

  async function changeRole(memberId: string, role: TeamRole) {
    if (!selectedWorkspaceId) return
    try {
      await invoke('team_update_member_role', {
        workspaceId: selectedWorkspaceId,
        memberId,
        role,
      })
      await loadWorkspaces()
    } catch (e) {
      error = String(e)
    }
  }

  let selectedWorkspace = $derived(workspaces.find((w) => w.id === selectedWorkspaceId))

  function selectWorkspace(id: string) {
    selectedWorkspaceId = id
    loadActivity()
  }

  loadWorkspaces()
</script>

<div class="flex h-full flex-col">
  <!-- Toolbar -->
  <div class="flex items-center gap-3 border-b border-border p-3">
    <Users class="h-5 w-5 text-text-muted" />
    <span class="text-sm font-medium">Team Workflows</span>
    <span class="text-xs text-text-muted">{workspaces.length} workspace(s)</span>
    <div class="flex-1"></div>
    <button
      class="flex items-center gap-2 rounded-md bg-accent px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover"
      onclick={() => (showCreateForm = !showCreateForm)}
    >
      <Plus class="h-4 w-4" />
      New Workspace
    </button>
  </div>

  {#if error}
    <div class="border-b border-border bg-error/10 p-3 text-sm text-error">{error}</div>
  {/if}
  {#if success}
    <div class="border-b border-border bg-success/10 p-3 text-sm text-success">{success}</div>
  {/if}

  {#if showCreateForm}
    <div class="border-b border-border bg-surface p-4">
      <h3 class="mb-3 text-sm font-medium">Create Workspace</h3>
      <div class="flex flex-col gap-3">
        <input
          type="text"
          class="rounded border border-border bg-bg px-3 py-2 text-sm outline-none focus:border-accent"
          placeholder="Workspace name"
          bind:value={newWorkspaceName}
        />
        <input
          type="text"
          class="rounded border border-border bg-bg px-3 py-2 text-sm outline-none focus:border-accent"
          placeholder="Description (optional)"
          bind:value={newWorkspaceDesc}
        />
        <div class="flex gap-2">
          <button
            class="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover"
            onclick={createWorkspace}
            disabled={!newWorkspaceName.trim()}
          >
            Create
          </button>
          <button
            class="rounded-md border border-border bg-bg px-4 py-2 text-sm transition-colors hover:bg-surface-hover"
            onclick={() => (showCreateForm = false)}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if workspaces.length === 0 && !loading}
    <div class="flex flex-1 items-center justify-center text-text-muted">
      <div class="text-center">
        <Users class="mx-auto mb-3 h-12 w-12 opacity-30" />
        <p class="text-sm">No workspaces yet.</p>
        <p class="mt-1 text-xs">Create a workspace to start collaborating with your team.</p>
      </div>
    </div>
  {:else if workspaces.length > 0}
    <div class="flex flex-1 overflow-hidden">
      <!-- Workspace List -->
      <div class="w-56 border-r border-border overflow-y-auto">
        {#each workspaces as ws (ws.id)}
          <button
            class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors {selectedWorkspaceId === ws.id ? 'bg-surface-hover text-text' : 'text-text-muted hover:text-text'}"
            onclick={() => selectWorkspace(ws.id)}
          >
            <Users class="h-4 w-4 flex-shrink-0" />
            <div class="flex-1 truncate">
              <div>{ws.name}</div>
              <div class="text-xs text-text-muted">{ws.members.length} member(s)</div>
            </div>
          </button>
        {/each}
      </div>

      <!-- Workspace Detail -->
      <div class="flex-1 overflow-y-auto p-4">
        {#if selectedWorkspace}
          <div class="mb-4">
            <div class="flex items-center gap-2">
              <h2 class="text-lg font-semibold">{selectedWorkspace.name}</h2>
              <button
                class="text-text-muted hover:text-error"
                onclick={() => deleteWorkspace(selectedWorkspace.id)}
                title="Delete workspace"
              >
                <Trash2 class="h-4 w-4" />
              </button>
            </div>
            {#if selectedWorkspace.description}
              <p class="mt-1 text-sm text-text-muted">{selectedWorkspace.description}</p>
            {/if}
          </div>

          <!-- Members -->
          <div class="mb-6 rounded-lg border border-border bg-surface p-4">
            <div class="mb-3 flex items-center justify-between">
              <h3 class="text-sm font-medium">Members ({selectedWorkspace.members.length})</h3>
              <button
                class="flex items-center gap-1 text-xs text-accent hover:text-accent-hover"
                onclick={() => (showAddMember = !showAddMember)}
              >
                <UserPlus class="h-3.5 w-3.5" />
                Invite
              </button>
            </div>

            {#if showAddMember}
              <div class="mb-3 flex flex-col gap-2 rounded border border-border bg-bg p-3">
                <input
                  type="text"
                  class="rounded border border-border bg-surface px-3 py-1.5 text-sm outline-none focus:border-accent"
                  placeholder="Name"
                  bind:value={newMemberName}
                />
                <input
                  type="email"
                  class="rounded border border-border bg-surface px-3 py-1.5 text-sm outline-none focus:border-accent"
                  placeholder="email@example.com"
                  bind:value={newMemberEmail}
                />
                <select
                  class="rounded border border-border bg-surface px-3 py-1.5 text-sm outline-none focus:border-accent"
                  bind:value={newMemberRole}
                >
                  <option value="admin">Admin</option>
                  <option value="editor">Editor</option>
                  <option value="viewer">Viewer</option>
                </select>
                <div class="flex gap-2">
                  <button
                    class="rounded bg-accent px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover"
                    onclick={addMember}
                    disabled={!newMemberName.trim() || !newMemberEmail.trim()}
                  >
                    Add Member
                  </button>
                  <button
                    class="rounded border border-border bg-surface px-3 py-1.5 text-xs transition-colors hover:bg-surface-hover"
                    onclick={() => (showAddMember = false)}
                  >
                    Cancel
                  </button>
                </div>
              </div>
            {/if}

            <div class="space-y-2">
              {#each selectedWorkspace.members as member (member.id)}
                {@const RoleIcon = roleIcons[member.role]}
                <div class="flex items-center gap-3 rounded border border-border bg-bg p-2">
                  <div
                    class="flex h-8 w-8 items-center justify-center rounded-full text-xs font-bold text-white"
                    style="background-color: {member.avatar_color}"
                  >
                    {member.name.charAt(0).toUpperCase()}
                  </div>
                  <div class="flex-1">
                    <div class="text-sm font-medium">{member.name}</div>
                    <div class="text-xs text-text-muted">{member.email}</div>
                  </div>
                  <div class="flex items-center gap-2">
                    <RoleIcon class="h-4 w-4 {roleColors[member.role]}" />
                    {#if member.role !== 'owner'}
                      <select
                        class="rounded border border-border bg-surface px-2 py-1 text-xs outline-none focus:border-accent"
                        value={member.role}
                        onchange={(e) => changeRole(member.id, e.currentTarget.value as TeamRole)}
                      >
                        <option value="admin">Admin</option>
                        <option value="editor">Editor</option>
                        <option value="viewer">Viewer</option>
                      </select>
                      <button
                        class="text-text-muted hover:text-error"
                        onclick={() => removeMember(member.id)}
                      >
                        <Trash2 class="h-3.5 w-3.5" />
                      </button>
                    {:else}
                      <span class="text-xs {roleColors[member.role]}">Owner</span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </div>

          <!-- Shared Collections -->
          <div class="mb-6 rounded-lg border border-border bg-surface p-4">
            <h3 class="mb-3 flex items-center gap-2 text-sm font-medium">
              <Share2 class="h-4 w-4 text-text-muted" />
              Shared Collections ({selectedWorkspace.collection_ids.length})
            </h3>
            {#if selectedWorkspace.collection_ids.length === 0}
              <p class="text-xs text-text-muted">No collections shared with this workspace yet.</p>
            {:else}
              <div class="space-y-1">
                {#each selectedWorkspace.collection_ids as colId}
                  <div class="flex items-center gap-2 rounded border border-border bg-bg p-2 text-sm">
                    <span class="font-mono text-xs text-text-muted">{colId}</span>
                    <div class="flex-1"></div>
                    <button
                      class="text-text-muted hover:text-error"
                      onclick={() => invoke('team_unshare_collection', { workspaceId: selectedWorkspace.id, collectionId: colId }).then(() => loadWorkspaces())}
                    >
                      <Trash2 class="h-3.5 w-3.5" />
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Activity Feed -->
          <div class="rounded-lg border border-border bg-surface p-4">
            <h3 class="mb-3 flex items-center gap-2 text-sm font-medium">
              <Activity class="h-4 w-4 text-text-muted" />
              Recent Activity
            </h3>
            {#if activities.length === 0}
              <p class="text-xs text-text-muted">No recent activity.</p>
            {:else}
              <div class="space-y-2">
                {#each activities as event (event.id)}
                  <div class="flex items-center gap-2 text-xs">
                    <div
                      class="flex h-6 w-6 items-center justify-center rounded-full text-[10px] font-bold text-white"
                      style="background-color: #6366f1"
                    >
                      {event.user_name.charAt(0).toUpperCase()}
                    </div>
                    <span class="text-text-muted">
                      <span class="font-medium text-text">{event.user_name}</span>
                      {' '}{event.action.replace(/_/g, ' ')}{' '}
                      <span class="font-mono">{event.resource_type}</span>
                      {' '}— {event.resource_name}
                    </span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {:else}
          <div class="flex h-full items-center justify-center text-text-muted">
            <p>Select a workspace to view details.</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
