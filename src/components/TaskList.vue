<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const refreshInterval = ref(null)
const currentTab = ref('processes')
const tabs = ['processes', 'usage', 'startup']
const processData = ref([])
const expandedGroups = ref([])

const isAnyExpanded = computed(() => expandedGroups.value.length > 0)

const sortConfig = ref({
 key: 'cpu_usage',
 desc: true
})

const formatMemory = (mbValue) => {
  const mb = Number(mbValue)
  if (mb >= 1024) return `${(mb / 1024).toFixed(2)} GB`
  if (mb < 1) return `${(mb * 1024).toFixed(0)} KB`
  return `${mb.toFixed(1)} MB`
}

const sortedProcesses = computed(() =>
 [...processData.value].sort((a, b) => {
   const modifier = sortConfig.value.desc ? -1 : 1
   const aVal = sortConfig.value.key === 'cpu_usage' ? a.total_cpu : a.total_memory
   const bVal = sortConfig.value.key === 'cpu_usage' ? b.total_cpu : b.total_memory
   return modifier * (aVal - bVal)
 })
)

const toggleGroup = (groupId) => {
 const idx = expandedGroups.value.indexOf(groupId)
 if (idx === -1) {
   expandedGroups.value.push(groupId)
   clearInterval(refreshInterval.value)
 } else {
   expandedGroups.value.splice(idx, 1)
   if (!isAnyExpanded.value) {
     startRefreshInterval()
   }
 }
}

const refreshProcesses = async () => {
 try {
   processData.value = await invoke('get_processes')
 } catch (err) {
   console.error('Failed to refresh:', err)
 }
}

const startRefreshInterval = () => {
 if (!isAnyExpanded.value) {
   refreshInterval.value = setInterval(refreshProcesses, 1000)
 }
}

const killProcess = async (pid) => {
 try {
   await invoke('kill_process', { pid })
   await refreshProcesses()
 } catch (err) {
   console.error('Failed to kill process:', err)
 }
}

onMounted(async () => {
 await refreshProcesses()
 startRefreshInterval()
})

onUnmounted(() => {
 clearInterval(refreshInterval.value)
})
</script>

<template>
    <div class="task-manager">
      <div class="tabs">
        <button v-for="tab in tabs" :key="tab" :class="{ active: currentTab === tab }" @click="currentTab = tab">
          {{ tab.charAt(0).toUpperCase() + tab.slice(1) }}
        </button>
        <button @click="refreshProcesses">Refresh</button>
      </div>

      <div v-if="currentTab === 'processes'" class="table-container">
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th @click="sortConfig = {key: 'cpu_usage', desc: !sortConfig.desc}">CPU</th>
              <th @click="sortConfig = {key: 'memory_usage', desc: !sortConfig.desc}">Memory</th>
              <th>Disk</th>
              <th>Network</th>
              <th>PID</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            <template v-for="group in sortedProcesses" :key="group.id">
              <tr :class="{ expanded: expandedGroups.includes(group.id) }">
                <td>
                  <span class="cursor-pointer" @click="toggleGroup(group.id)">
                    {{ group.children.length ? (expandedGroups.includes(group.id) ? '▼' : '▶') : '•' }}
                    {{ group.name }}
                  </span>
                </td>
                <td>{{ group.total_cpu }}%</td>
                <td>{{ formatMemory(group.total_memory) }}</td>
                <td>-</td>
                <td>-</td>
                <td>{{ group.id }}</td>
                <td><button class="kill-btn" @click="killProcess(group.id)">Kill</button></td>
              </tr>
              <template v-if="expandedGroups.includes(group.id)">
                <tr v-for="child in group.children" :key="child.id" class="child-process">
                  <td class="pl-8">{{ child.name }}</td>
                  <td>{{ child.cpu_usage }}%</td>
                  <td>{{ formatMemory(child.memory_usage) }}</td>
                  <td>-</td>
                  <td>-</td>
                  <td>{{ child.id }}</td>
                  <td><button class="kill-btn" @click="killProcess(child.id)">Kill</button></td>
                </tr>
              </template>
            </template>
          </tbody>
        </table>
      </div>
    </div>
</template>

<style scoped>
.task-manager {
  background: #0a0a0a;
  color: #e4e4e7;
  min-height: 100vh;
  padding: 1.5rem;
  font-family: system-ui, -apple-system, sans-serif;
}

.tabs {
  display: flex;
  gap: 1rem;
  margin-bottom: 2rem;
  background: #18181b;
  padding: 0.75rem;
  border-radius: 8px;
}

.tabs button {
  background: #27272a;
  color: #e4e4e7;
  border: none;
  padding: 0.625rem 1.25rem;
  border-radius: 6px;
  font-weight: 500;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.tabs button:hover {
  background: #3f3f46;
  transform: translateY(-1px);
}

.tabs button.active {
  background: #B31A57;
  color: #fff;
  box-shadow: 0 1px 3px rgba(179, 26, 87, 0.3);
}

.table-container {
  background: #18181b;
  border-radius: 12px;
  padding: 1rem;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
}

table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  text-align: left;
}

th {
  color: #B31A57;
  font-weight: 600;
  padding: 1rem;
  border-bottom: 1px solid #27272a;
  cursor: pointer;
  transition: color 0.2s;
}

th:hover {
  color: #d33172;
}

td {
  padding: 1rem;
  border-bottom: 1px solid #27272a;
}

tr {
  transition: background-color 0.2s;
}

tr.expanded {
  background: #b31a5769;
  border-bottom: 1px solid #B31A57;
}

tr:hover:not(.expanded) {
  background: #27272a;
}

.child-process:hover {
  background: #641233;
}

.kill-btn {
  background: #27272a;
  color: #ef4444;
  border: 1px solid #ef4444;
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
  border-radius: 6px;
  transition: all 0.2s;
  opacity: 0;
}

tr:hover .kill-btn {
  opacity: 1;
}

.kill-btn:hover {
  background: #ef4444;
  color: #fff;
}

.cursor-pointer {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.pl-8 {
  padding-left: 2.5rem;
}

.child-process {
  background: rgba(179, 26, 87, 0.05);
}

.child-process:hover {
  background: rgba(179, 26, 87, 0.1);
}
</style>