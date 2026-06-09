<script setup lang="ts">
import { ref, reactive } from "vue";

const mns = ref([
  { name: "VPS-1", ipAddress: "8526:efcf:c8ca:8cb4:82bf:ab6d:5504:9299", status: "ENABLED" },
  { name: "VPS-2", ipAddress: "123.234.456.124:1234", status: "PRE_ENABLED" },
  { name: "My VPS test", ipAddress: "127.0.0.1", status: "MISSING" },
]);

const form = reactive({ name: "", ipAddress: "", status: "ENABLED" });
const showForm = ref(false);

function badgeClass(status: string) {
  return (
    { ENABLED: "bg-success", PRE_ENABLED: "bg-warning text-dark", MISSING: "bg-danger" }[status] ?? "bg-secondary"
  );
}

function badgeLabel(status: string) {
  return { ENABLED: "Enabled", PRE_ENABLED: "Pre-enabled", MISSING: "Missing" }[status] ?? status;
}

function addMn() {
  if (!form.name || !form.ipAddress) return;
  mns.value.push({ ...form });
  form.name = "";
  form.ipAddress = "";
  form.status = "ENABLED";
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
        <span :class="['badge', badgeClass(mn.status)]">{{ badgeLabel(mn.status) }}</span>
      </div>
    </div>

    <div v-if="showForm" class="card mb-3">
      <div class="card-body d-flex flex-column gap-2">
        <input v-model="form.name" class="form-control form-control-sm" placeholder="Name" />
        <input v-model="form.ipAddress" class="form-control form-control-sm" placeholder="IP address" />
        <select v-model="form.status" class="form-select form-select-sm">
          <option value="ENABLED">Enabled</option>
          <option value="PRE_ENABLED">Pre-enabled</option>
          <option value="MISSING">Missing</option>
        </select>
        <div class="d-flex gap-2">
          <button class="btn btn-primary btn-sm" @click="addMn">Add</button>
          <button class="btn btn-outline-secondary btn-sm" @click="showForm = false">Cancel</button>
        </div>
      </div>
    </div>

    <button v-if="!showForm" class="btn btn-outline-primary btn-sm" @click="showForm = true">
	+ Add node
    </button>
  </div>
</template>
<style scoped>
</style>
