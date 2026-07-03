<script setup lang="ts">
import { computed, ref, watch, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { Button } from "@/components/ui/button";
import { Plus, Trash2, ArrowRight } from "@lucide/vue";
import SearchableSelect from "@/components/ui/searchable-select/SearchableSelect.vue";
import { listDialectDataTypes } from "@/lib/api";
import { getDataTypeOptions } from "@/lib/tableStructureEditorState";
import type { FieldMappingEntry } from "@/types/schemaDiff";

const { t } = useI18n();

const props = defineProps<{
  mappings: FieldMappingEntry[];
  sourceDbType: string;
  targetDbType: string;
}>();

const emit = defineEmits<{
  (e: "update:mappings", value: FieldMappingEntry[]): void;
}>();

const sameType = computed(() => props.sourceDbType === props.targetDbType);

const sourceTypeOptions = ref<string[]>([]);
const targetTypeOptions = ref<string[]>([]);

async function loadSourceTypes() {
  try {
    const types = await listDialectDataTypes(props.sourceDbType);
    sourceTypeOptions.value = types.length > 0 ? types : getDataTypeOptions(props.sourceDbType as any);
  } catch {
    sourceTypeOptions.value = getDataTypeOptions(props.sourceDbType as any);
  }
}

async function loadTargetTypes() {
  try {
    const types = await listDialectDataTypes(props.targetDbType);
    targetTypeOptions.value = types.length > 0 ? types : getDataTypeOptions(props.targetDbType as any);
  } catch {
    targetTypeOptions.value = getDataTypeOptions(props.targetDbType as any);
  }
}

function addMapping() {
  emit("update:mappings", [...props.mappings, { sourceType: "", targetType: "" }]);
}

function removeMapping(index: number) {
  const next = props.mappings.filter((_, i) => i !== index);
  emit("update:mappings", next);
}

function updateMapping(index: number, field: "sourceType" | "targetType", value: string) {
  const next = props.mappings.map((m, i) => (i === index ? { ...m, [field]: value } : m));
  emit("update:mappings", next);
}

watch(() => props.sourceDbType, loadSourceTypes);
watch(() => props.targetDbType, loadTargetTypes);

onMounted(() => {
  loadSourceTypes();
  loadTargetTypes();
});
</script>

<template>
  <div class="border rounded-lg bg-card">
    <div class="flex items-center gap-2 px-3 py-2 border-b">
      <span class="text-xs font-medium">{{ t("diff.fieldMapping.title") }}</span>
      <span v-if="!sameType" class="flex items-center gap-1 ml-1">
        <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300">{{ sourceDbType }}</span>
        <ArrowRight class="w-3 h-3 text-muted-foreground" />
        <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300">{{ targetDbType }}</span>
      </span>
    </div>

    <div v-if="sameType" class="px-3 py-4 text-xs text-muted-foreground text-center">
      {{ t("diff.fieldMapping.sameTypeHint") }}
    </div>

    <div v-else class="p-3">
      <div v-if="mappings.length === 0" class="flex flex-col items-center justify-center py-6 gap-2">
        <p class="text-xs text-muted-foreground">{{ t("diff.fieldMapping.noMappings") }}</p>
      </div>

      <div v-else class="space-y-2">
        <div class="grid grid-cols-[1fr_auto_1fr_auto] gap-2 items-center px-1">
          <span class="text-[10px] font-medium text-muted-foreground">{{ t("diff.fieldMapping.sourceType") }}</span>
          <div />
          <span class="text-[10px] font-medium text-muted-foreground">{{ t("diff.fieldMapping.targetType") }}</span>
          <div />
        </div>
        <div v-for="(mapping, i) in mappings" :key="i" class="grid grid-cols-[1fr_auto_1fr_auto] gap-2 items-center">
          <SearchableSelect
            :model-value="mapping.sourceType"
            @update:model-value="(v: string) => updateMapping(i, 'sourceType', v)"
            :options="sourceTypeOptions"
            :placeholder="t('diff.fieldMapping.sourceType')"
            :search-placeholder="t('diff.fieldMapping.sourceType')"
            :empty-text="t('common.noResults')"
            trigger-variant="outline"
            trigger-class="h-8 w-full justify-between text-xs font-mono"
            content-class="w-[var(--reka-popover-trigger-width)]"
            allow-custom
          />
          <ArrowRight class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
          <SearchableSelect
            :model-value="mapping.targetType"
            @update:model-value="(v: string) => updateMapping(i, 'targetType', v)"
            :options="targetTypeOptions"
            :placeholder="t('diff.fieldMapping.targetType')"
            :search-placeholder="t('diff.fieldMapping.targetType')"
            :empty-text="t('common.noResults')"
            trigger-variant="outline"
            trigger-class="h-8 w-full justify-between text-xs font-mono"
            content-class="w-[var(--reka-popover-trigger-width)]"
            allow-custom
          />
          <Button variant="ghost" size="icon-sm" class="h-8 w-8 shrink-0 text-muted-foreground hover:text-destructive" @click="removeMapping(i)">
            <Trash2 class="w-3.5 h-3.5" />
          </Button>
        </div>
      </div>

      <div class="mt-3">
        <Button variant="outline" size="sm" class="h-7 text-xs gap-1" @click="addMapping">
          <Plus class="w-3.5 h-3.5" />
          {{ t("diff.fieldMapping.addMapping") }}
        </Button>
      </div>
    </div>
  </div>
</template>
