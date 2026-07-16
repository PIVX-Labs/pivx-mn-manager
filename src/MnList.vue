<script setup lang="ts">
import { ref, reactive, Ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { VPS } from "./types/vps";
import {
  Masternode,
  MasternodeStatuses,
  getMasternodeStatus,
} from "./types/masternode";

const vps: Ref<VPS[]> = ref([]);

const form = reactive({
  name: "",
  ipAddress: "",
  authentication: "password" as "password" | "key",
  username: "",
  password: "",
  privateKeyPath: "",
});

// One "add masternode" form per VPS, keyed by VPS ipAddress
const mnForms = reactive<
  Record<
    string,
    {
      name: string;
      ipAddress: string;
      privateKeyPath: string;
      collateralTxId: string;
      outId: number;
    }
  >
>({});

type StatusEntry = MasternodeStatuses | "LOADING" | "ERROR";
const statuses = reactive<Record<string, StatusEntry>>({});

const showForm = ref(false);
const showMnForm = ref<Record<string, boolean>>({});
const saveError = ref<string | null>(null);

function statusKey(s: VPS, mn: Masternode) {
  return `${s.ipAddress}::${mn.ipAddress}`;
}

function badgeClass(status: string) {
  return (
    {
      ENABLED: "bg-success",
      PRE_ENABLED: "bg-warning text-dark",
      MISSING: "bg-danger",
      MISSING_COLLATERAL: "bg-danger",
      LOADING: "bg-secondary",
      ERROR: "bg-dark",
    }[status] ?? "bg-secondary"
  );
}

function badgeLabel(status: string) {
  return (
    {
      ENABLED: "Enabled",
      PRE_ENABLED: "Pre-enabled",
      MISSING: "Missing",
      MISSING_COLLATERAL: "Missing collateral",
      LOADING: "Checking…",
      ERROR: "Error",
    }[status] ?? status
  );
}

async function refreshStatus(s: VPS, mn: Masternode) {
  const key = statusKey(s, mn);
  statuses[key] = "LOADING";
  try {
    statuses[key] = await getMasternodeStatus(mn);
  } catch {
    statuses[key] = "ERROR";
  }
}

async function refreshVpsStatuses(s: VPS) {
  await Promise.all(s.getMasternodes().map((mn) => refreshStatus(s, mn)));
}

function ensureMnForm(s: VPS) {
  if (!mnForms[s.ipAddress]) {
    mnForms[s.ipAddress] = {
      name: "",
      ipAddress: "",
      privateKeyPath: "",
      collateralTxId: "",
      outId: 0,
    };
  }
  return mnForms[s.ipAddress];
}

function toggleMnForm(s: VPS) {
  ensureMnForm(s);
  showMnForm.value[s.ipAddress] = !showMnForm.value[s.ipAddress];
}

async function addMasternode(s: VPS) {
  const f = ensureMnForm(s);
  if (!f.name || !f.ipAddress) return;
  const mn: Masternode = {
    name: f.name,
    ipAddress: f.ipAddress,
    privateKey: f.privateKeyPath,
    collateralTxId: f.collateralTxId,
    outId: f.outId,
  };
  s.addMasternode(mn);
  f.name = "";
  f.ipAddress = "";
  f.privateKeyPath = "";
  f.collateralTxId = "";
  showMnForm.value[s.ipAddress] = false;
  await refreshStatus(s, mn);
}

async function saveMns() {
  saveError.value = null;
  try {
    const path = await save({
      defaultPath: "masternodes.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path) return; // user cancelled
    const payload = vps.value.map((s) => ({
      name: s.name,
      authentication: s.authentication,
      username: s.username,
      ipAddress: s.ipAddress,
      masternodes: s.getMasternodes(),
    }));
    await writeTextFile(path, JSON.stringify(payload, null, 2));
  } catch (e) {
    saveError.value = String(e);
  }
}

async function addVPN() {
  if (!form.name || !form.ipAddress) return;
  const server = new VPS(
    form.name,
    form.authentication,
    form.username,
    form.password || form.privateKeyPath,
    form.ipAddress,
    [],
  );
  //await server.setupVps();
  vps.value.push(server);
  showForm.value = false;
  form.name = "";
  form.ipAddress = "";
  form.username = "";
  form.password = "";
  form.privateKeyPath = "";
}
</script>

<template>
  <div class="container py-3">
    <div v-if="saveError" class="alert alert-danger py-1 px-2 small mb-2">
      {{ saveError }}
    </div>

    <div v-if="vps.length === 0" class="text-muted small mb-3">
      No VPS servers yet. Add one to get started.
    </div>

    <div v-for="s in vps" :key="s.ipAddress" class="card mb-3">
      <div class="card-header d-flex align-items-center gap-3">
        <div class="flex-grow-1">
          <div class="fw-semibold">{{ s.name }}</div>
          <code class="text-muted small">{{ s.ipAddress }}</code>
        </div>
        <span class="badge bg-light text-dark border">
          {{ s.getMasternodes().length }} masternode{{
            s.getMasternodes().length === 1 ? "" : "s"
          }}
        </span>
        <button
          class="btn btn-outline-secondary btn-sm"
          title="Refresh statuses"
          @click="refreshVpsStatuses(s)"
        >
          ↻
        </button>
      </div>

      <ul class="list-group list-group-flush">
        <li
          v-for="mn in s.getMasternodes()"
          :key="mn.ipAddress"
          class="list-group-item d-flex align-items-center gap-3"
        >
          <div class="flex-grow-1">
            <div class="fw-semibold">{{ mn.name }}</div>
            <code class="text-muted small">{{ mn.ipAddress }}</code>
            <div v-if="mn.collateralTxId" class="text-muted small">
              tx: {{ mn.collateralTxId }}
            </div>
          </div>
          <span
            :class="[
              'badge',
              badgeClass(statuses[statusKey(s, mn)] ?? 'MISSING'),
            ]"
          >
            {{ badgeLabel(statuses[statusKey(s, mn)] ?? "MISSING") }}
          </span>
        </li>
        <li
          v-if="s.getMasternodes().length === 0"
          class="list-group-item text-muted small"
        >
          No masternodes on this VPS yet.
        </li>
      </ul>

      <div class="card-body">
        <div
          v-if="showMnForm[s.ipAddress]"
          class="d-flex flex-column gap-2 mb-2"
        >
          <input
            v-model="mnForms[s.ipAddress].name"
            class="form-control form-control-sm"
            placeholder="Masternode name"
          />
          <input
            v-model="mnForms[s.ipAddress].ipAddress"
            class="form-control form-control-sm"
            placeholder="Masternode IP address"
          />
          <input
            v-model="mnForms[s.ipAddress].collateralTxId"
            class="form-control form-control-sm"
            placeholder="Collateral transaction ID"
          />
          <input
            v-model="mnForms[s.ipAddress].collateralTxId"
            class="form-control form-control-sm"
            placeholder="Out id"
            type="number"
          />
          <input
            class="form-control form-control-sm"
            type="file"
            placeholder="Private key file"
            v-on:change="
              (event: Event) =>
                (mnForms[s.ipAddress].privateKeyPath =
                  (event.target as HTMLInputElement).files?.[0]?.name ?? '')
            "
          />
          <div class="d-flex gap-2">
            <button class="btn btn-primary btn-sm" @click="addMasternode(s)">
              Add masternode
            </button>
            <button
              class="btn btn-outline-secondary btn-sm"
              @click="showMnForm[s.ipAddress] = false"
            >
              Cancel
            </button>
          </div>
        </div>
        <button
          v-else
          class="btn btn-outline-primary btn-sm"
          @click="toggleMnForm(s)"
        >
          + Add masternode
        </button>
      </div>
    </div>

    <div v-if="showForm" class="card mb-3">
      <div class="card-body d-flex flex-column gap-2">
        <input
          v-model="form.name"
          class="form-control form-control-sm"
          placeholder="Name"
        />
        <input
          v-model="form.ipAddress"
          class="form-control form-control-sm"
          placeholder="IP address"
        />
        <select
          v-model="form.authentication"
          class="form-select form-select-sm"
        >
          <option value="password">Password</option>
          <option value="key">Key authentication</option>
        </select>
        <input
          v-model="form.username"
          class="form-control form-control-sm"
          placeholder="Username"
        />
        <input
          v-if="form.authentication === 'password'"
          v-model="form.password"
          class="form-control form-control-sm"
          placeholder="Password"
          type="password"
        />
        <input
          v-if="form.authentication === 'key'"
          class="form-control form-control-sm"
          type="file"
          placeholder="Private key file"
          v-on:change="
            (event: Event) =>
              (form.privateKeyPath =
                (event.target as HTMLInputElement).files?.[0]?.name ?? '')
          "
        />
        <div class="d-flex gap-2">
          <button class="btn btn-primary btn-sm" @click="addVPN">Add</button>
          <button
            class="btn btn-outline-secondary btn-sm"
            @click="showForm = false"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>

    <div class="d-flex gap-2">
      <button
        v-if="!showForm"
        class="btn btn-outline-primary btn-sm"
        @click="showForm = true"
      >
        + Add VPS
      </button>
      <button
        class="btn btn-outline-primary btn-sm"
        @click="saveMns"
        :disabled="vps.length === 0"
      >
        Save to file
      </button>
    </div>
  </div>
</template>
