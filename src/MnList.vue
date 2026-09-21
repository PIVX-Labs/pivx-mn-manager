<script setup lang="ts">
import { ref, reactive, computed, onMounted, Ref } from "vue";
import { save, open } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { VPS } from "./types/vps";
import {
  Masternode,
  MasternodeStatuses,
  getMasternodeStatus,
} from "./types/masternode";
import { openUrl } from "@tauri-apps/plugin-opener";

const vps: Ref<VPS[]> = ref([]);
const BASE_MPW_URL = "https://app.mypivxwallet.org";

const IP_REGEX =
  /^(25[0-5]|2[0-4]\d|1\d{2}|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d{2}|[1-9]?\d)){3}$/;

const form = reactive({
  name: "",
  ipAddress: "",
  authentication: "password" as "password" | "key",
  username: "",
  password: "",
  privateKeyPath: "",
});
const addingVps = ref(false);
const vpsError = ref<string | null>(null);

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
const mnErrors = reactive<Record<string, string | null>>({});
const addingMn = reactive<Record<string, boolean>>({});

type StatusEntry = MasternodeStatuses | "LOADING" | "ERROR";
const statuses = reactive<Record<string, StatusEntry>>({});

const showForm = ref(false);
const showMnForm = ref<Record<string, boolean>>({});
const saveError = ref<string | null>(null);
const saveSuccess = ref(false);
const isSaving = ref(false);
const refreshingVps = reactive<Record<string, boolean>>({});

const STATUS_META: Record<string, { label: string; badgeClass: string }> = {
  ENABLED: { label: "Enabled", badgeClass: "bg-success" },
  PRE_ENABLED: { label: "Pre-enabled", badgeClass: "bg-warning text-dark" },
  MISSING: { label: "Missing", badgeClass: "bg-danger" },
  MISSING_COLLATERAL: {
    label: "Missing collateral",
    badgeClass: "bg-danger",
  },
  LOADING: { label: "Checking…", badgeClass: "bg-secondary" },
  ERROR: { label: "Error", badgeClass: "bg-dark" },
};

function statusKey(s: VPS, mn: Masternode) {
  return `${s.ipAddress}::${mn.ipAddress}`;
}

function statusMeta(s: VPS, mn: Masternode) {
  const status = statuses[statusKey(s, mn)] ?? "MISSING";
  return STATUS_META[status] ?? { label: status, badgeClass: "bg-secondary" };
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
  refreshingVps[s.ipAddress] = true;
  try {
    await Promise.all(s.getMasternodes().map((mn) => refreshStatus(s, mn)));
  } finally {
    refreshingVps[s.ipAddress] = false;
  }
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
  mnErrors[s.ipAddress] = null;
  showMnForm.value[s.ipAddress] = !showMnForm.value[s.ipAddress];
}

async function pickPrivateKey(target: { privateKeyPath: string }) {
  const selected = await open({
    multiple: false,
    title: "Select private key file",
  });
  if (typeof selected === "string") {
    target.privateKeyPath = selected;
  }
}

function validateMnForm(s: VPS, f: (typeof mnForms)[string]): string | null {
  if (!f.name.trim()) return "Masternode name is required.";
  if (!f.ipAddress.trim() || !IP_REGEX.test(f.ipAddress.trim())) {
    return "Enter a valid IP address for the masternode.";
  }
  if (s.getMasternodes().some((mn) => mn.ipAddress === f.ipAddress.trim())) {
    return "A masternode with this IP already exists on this VPS.";
  }
  return null;
}

async function addMasternode(s: VPS) {
  const f = ensureMnForm(s);
  const error = validateMnForm(s, f);
  mnErrors[s.ipAddress] = error;
  if (error) return;

  addingMn[s.ipAddress] = true;
  try {
    const mn: Masternode = {
      name: f.name.trim(),
      ipAddress: f.ipAddress.trim(),
      privateKey: f.privateKeyPath,
      collateralTxId: f.collateralTxId.trim(),
      outId: f.outId,
    };
    s.addMasternode(mn);
    f.name = "";
    f.ipAddress = "";
    f.privateKeyPath = "";
    f.collateralTxId = "";
    f.outId = 0;
    showMnForm.value[s.ipAddress] = false;
    await refreshStatus(s, mn);
  } finally {
    addingMn[s.ipAddress] = false;
  }
}

function removeMasternode(s: VPS, mn: Masternode) {
  if (
    !confirm(
      `Remove masternode "${mn.name}" (${mn.ipAddress})? This can't be undone.`,
    )
  ) {
    return;
  }
  s.removeMasternode(mn.ipAddress);
  delete statuses[statusKey(s, mn)];
}

function removeVps(s: VPS) {
  if (
    !confirm(
      `Remove VPS "${s.name}" and all ${s.getMasternodes().length} masternode(s) on it? This can't be undone.`,
    )
  ) {
    return;
  }
  vps.value = vps.value.filter((v) => v.ipAddress !== s.ipAddress);
  for (const mn of s.getMasternodes()) {
    delete statuses[statusKey(s, mn)];
  }
  delete mnForms[s.ipAddress];
  delete showMnForm.value[s.ipAddress];
}

async function saveMns() {
  saveError.value = null;
  saveSuccess.value = false;
  isSaving.value = true;
  try {
    const path = await save({
      defaultPath: "masternodes.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path) return;
    const payload = vps.value.map((s) => ({
      name: s.name,
      authentication: s.authentication,
      username: s.username,
      ipAddress: s.ipAddress,
      masternodes: s.getMasternodes(),
    }));
    await writeTextFile(path, JSON.stringify(payload, null, 2));
    saveSuccess.value = true;
  } catch (e) {
    saveError.value = String(e);
  } finally {
    isSaving.value = false;
  }
}

function validateVpsForm(): string | null {
  if (!form.name.trim()) return "Name is required.";
  if (!form.ipAddress.trim() || !IP_REGEX.test(form.ipAddress.trim())) {
    return "Enter a valid IP address.";
  }
  if (vps.value.some((v) => v.ipAddress === form.ipAddress.trim())) {
    return "A VPS with this IP address has already been added.";
  }
  if (!form.username.trim()) return "Username is required.";
  if (form.authentication === "password" && !form.password) {
    return "Password is required.";
  }
  if (form.authentication === "key" && !form.privateKeyPath) {
    return "Select a private key file.";
  }
  return null;
}

async function addVPN() {
  const error = validateVpsForm();
  vpsError.value = error;
  if (error) return;

  addingVps.value = true;
  try {
    const server = new VPS(
      form.name.trim(),
      form.authentication,
      form.username.trim(),
      form.authentication === "password" ? form.password : form.privateKeyPath,
      form.ipAddress.trim(),
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
  } finally {
    addingVps.value = false;
  }
}

async function openMpwTx() {
  await openUrl(`${BASE_MPW_URL}?pay=true&payYourself=true&amount=10000`);
}

const totalMasternodes = computed(() =>
  vps.value.reduce((sum, s) => sum + s.getMasternodes().length, 0),
);

onMounted(() => {
  vps.value.forEach(refreshVpsStatuses);
});
</script>

<template>
  <div class="container py-3">
    <div v-if="saveError" class="alert alert-danger py-1 px-2 small mb-2">
      {{ saveError }}
    </div>
    <div
      v-if="saveSuccess"
      class="alert alert-success py-1 px-2 small mb-2"
      role="status"
    >
      Saved.
    </div>

    <div v-if="vps.length === 0" class="text-muted small mb-3">
      No VPS servers yet. Add one to get started.
    </div>
    <div v-else class="text-muted small mb-3">
      {{ vps.length }} VPS{{ vps.length === 1 ? "" : "s" }},
      {{ totalMasternodes }} masternode{{ totalMasternodes === 1 ? "" : "s" }}
      total
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
          :disabled="refreshingVps[s.ipAddress]"
          @click="refreshVpsStatuses(s)"
        >
          <span
            v-if="refreshingVps[s.ipAddress]"
            class="spinner-border spinner-border-sm"
            aria-hidden="true"
          ></span>
          <span v-else>↻</span>
        </button>
        <button
          class="btn btn-outline-danger btn-sm"
          title="Remove VPS"
          @click="removeVps(s)"
        >
          Remove
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
          <span :class="['badge', statusMeta(s, mn).badgeClass]">
            {{ statusMeta(s, mn).label }}
          </span>
          <button
            class="btn btn-outline-danger btn-sm"
            title="Remove masternode"
            @click="removeMasternode(s, mn)"
          >
            ✕
          </button>
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
          <div
            v-if="mnErrors[s.ipAddress]"
            class="alert alert-danger py-1 px-2 small mb-0"
          >
            {{ mnErrors[s.ipAddress] }}
          </div>
          <label class="small text-muted mb-0" :for="`mn-name-${s.ipAddress}`"
            >Masternode name</label
          >
          <input
            :id="`mn-name-${s.ipAddress}`"
            v-model="mnForms[s.ipAddress].name"
            class="form-control form-control-sm"
            placeholder="Masternode name"
          />
          <label class="small text-muted mb-0" :for="`mn-ip-${s.ipAddress}`"
            >Masternode IP address</label
          >
          <input
            :id="`mn-ip-${s.ipAddress}`"
            v-model="mnForms[s.ipAddress].ipAddress"
            class="form-control form-control-sm"
            placeholder="e.g. 203.0.113.10"
          />
          <label class="small text-muted mb-0" :for="`mn-tx-${s.ipAddress}`"
            >Collateral transaction ID</label
          >
          <div class="d-flex gap-2">
            <input
              :id="`mn-tx-${s.ipAddress}`"
              v-model="mnForms[s.ipAddress].collateralTxId"
              class="form-control form-control-sm"
              placeholder="Collateral transaction ID"
            />
            <button
              class="btn btn-outline-secondary btn-sm text-nowrap"
              @click="openMpwTx()"
            >
              Pay via MPW
            </button>
          </div>
          <label class="small text-muted mb-0" :for="`mn-out-${s.ipAddress}`"
            >Out ID</label
          >
          <input
            :id="`mn-out-${s.ipAddress}`"
            v-model.number="mnForms[s.ipAddress].outId"
            class="form-control form-control-sm"
            placeholder="Out id"
            type="number"
            min="0"
          />
          <label class="small text-muted mb-0">Private key file</label>
          <div class="d-flex align-items-center gap-2">
            <button
              class="btn btn-outline-secondary btn-sm"
              @click="pickPrivateKey(mnForms[s.ipAddress])"
            >
              Choose file…
            </button>
            <code class="small text-muted">
              {{ mnForms[s.ipAddress].privateKeyPath || "No file selected" }}
            </code>
          </div>
          <div class="d-flex gap-2 mt-1">
            <button
              class="btn btn-primary btn-sm"
              :disabled="addingMn[s.ipAddress]"
              @click="addMasternode(s)"
            >
              <span
                v-if="addingMn[s.ipAddress]"
                class="spinner-border spinner-border-sm me-1"
                aria-hidden="true"
              ></span>
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
        <div v-if="vpsError" class="alert alert-danger py-1 px-2 small mb-0">
          {{ vpsError }}
        </div>
        <label class="small text-muted mb-0" for="vps-name">Name</label>
        <input
          id="vps-name"
          v-model="form.name"
          class="form-control form-control-sm"
          placeholder="Name"
        />
        <label class="small text-muted mb-0" for="vps-ip">IP address</label>
        <input
          id="vps-ip"
          v-model="form.ipAddress"
          class="form-control form-control-sm"
          placeholder="e.g. 203.0.113.5"
        />
        <label class="small text-muted mb-0" for="vps-auth"
          >Authentication</label
        >
        <select
          id="vps-auth"
          v-model="form.authentication"
          class="form-select form-select-sm"
        >
          <option value="password">Password</option>
          <option value="key">Key authentication</option>
        </select>
        <label class="small text-muted mb-0" for="vps-user">Username</label>
        <input
          id="vps-user"
          v-model="form.username"
          class="form-control form-control-sm"
          placeholder="Username"
        />
        <template v-if="form.authentication === 'password'">
          <label class="small text-muted mb-0" for="vps-pass">Password</label>
          <input
            id="vps-pass"
            v-model="form.password"
            class="form-control form-control-sm"
            placeholder="Password"
            type="password"
            autocomplete="new-password"
          />
        </template>
        <template v-else>
          <label class="small text-muted mb-0">Private key file</label>
          <div class="d-flex align-items-center gap-2">
            <button
              class="btn btn-outline-secondary btn-sm"
              @click="pickPrivateKey(form)"
            >
              Choose file…
            </button>
            <code class="small text-muted">
              {{ form.privateKeyPath || "No file selected" }}
            </code>
          </div>
        </template>
        <div class="d-flex gap-2 mt-1">
          <button
            class="btn btn-primary btn-sm"
            :disabled="addingVps"
            @click="addVPN"
          >
            <span
              v-if="addingVps"
              class="spinner-border spinner-border-sm me-1"
              aria-hidden="true"
            ></span>
            Add
          </button>
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
        :disabled="vps.length === 0 || isSaving"
        @click="saveMns"
      >
        <span
          v-if="isSaving"
          class="spinner-border spinner-border-sm me-1"
          aria-hidden="true"
        ></span>
        Save to file
      </button>
    </div>
  </div>
</template>
