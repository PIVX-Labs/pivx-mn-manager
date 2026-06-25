<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref, reactive } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";

const mns = ref([
  {
    name: "VPS-1",
    ipAddress: "8526:efcf:c8ca:8cb4:82bf:ab6d:5504:9299",
    status: "ENABLED",
  },
  { name: "VPS-2", ipAddress: "123.234.456.124:1234", status: "PRE_ENABLED" },
  { name: "My VPS test", ipAddress: "127.0.0.1", status: "MISSING" },
]);

const form = reactive({
  name: "",
  ipAddress: "",
  authentication: "password",
  username: "",
  password: "",
  privateKeyPath: "",
});

const showForm = ref(false);
const saveError = ref<string | null>(null);

function badgeClass(status: string) {
  return (
    {
      ENABLED: "bg-success",
      PRE_ENABLED: "bg-warning text-dark",
      MISSING: "bg-danger",
    }[status] ?? "bg-secondary"
  );
}

function badgeLabel(status: string) {
  return (
    { ENABLED: "Enabled", PRE_ENABLED: "Pre-enabled", MISSING: "Missing" }[
      status
    ] ?? status
  );
}

async function saveMns() {
  saveError.value = null;
  try {
    const path = await save({
      defaultPath: "masternodes.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path) return; // user cancelled
    await writeTextFile(path, JSON.stringify(mns.value, null, 2));
  } catch (e) {
    saveError.value = String(e);
  }
}

function addMn() {
  if (!form.name || !form.ipAddress) return;
  if (form.authentication === "password") {
    invoke("init_vps_with_password", {
      ip_address: form.ipAddress,
      username: form.username,
      password: form.password,
    });
  } else if (form.authentication === "key") {
    invoke("init_vps_with_key", {
      ip_address: form.ipAddress,
      username: form.username,
      private_key_path: form.privateKeyPath,
    });
  }
  mns.value.push({
    name: form.name,
    ipAddress: form.ipAddress,
    status: "PRE_ENABLED",
  });
  showForm.value = false;
}
</script>

<template>
  <div class="container py-3">
    <div class="list-group mb-3">
      <div
        v-for="mn in mns"
        :key="mn.ipAddress"
        class="list-group-item d-flex align-items-center gap-3"
      >
        <div class="flex-grow-1">
          <div class="fw-semibold">{{ mn.name }}</div>
          <code class="text-muted small">{{ mn.ipAddress }}</code>
        </div>
        <span :class="['badge', badgeClass(mn.status)]">{{
          badgeLabel(mn.status)
        }}</span>
      </div>
    </div>

    <div v-if="saveError" class="alert alert-danger py-1 px-2 small mb-2">
      {{ saveError }}
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
          <button class="btn btn-primary btn-sm" @click="addMn">Add</button>
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
        + Add node
      </button>
      <button
        class="btn btn-outline-primary btn-sm"
        @click="saveMns"
        :disabled="mns.length === 0"
      >
        Save to file
      </button>
    </div>
  </div>
</template>
