<script setup lang="ts">
    import { computed } from 'vue'

    const props = withDefaults(
        defineProps<{
            label: string
            align?: 'center' | 'left'
        }>(),
        {
            align: 'center',
        },
    )

    const segments = computed(() =>
        props.label
            .trim()
            .split(/\s+/u)
            .filter(Boolean)
            .map((segment) => segment.match(/[^/]*\/|[^/]+/gu) ?? [segment]),
    )
</script>

<template>
    <span class="key-label" :class="`key-label-${align}`">
        <span
            v-for="(parts, index) in segments"
            :key="`${parts.join('')}-${index}`"
            class="key-label-segment"
        >
            <span
                v-for="(part, partIndex) in parts"
                :key="`${part}-${partIndex}`"
                class="key-label-part"
            >
                {{ part }}
            </span>
        </span>
    </span>
</template>

<style scoped>
    .key-label {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        max-width: 100%;
        min-width: 0;
        column-gap: 0.25em;
        line-height: inherit;
    }

    .key-label-center {
        justify-content: center;
    }

    .key-label-left {
        justify-content: flex-start;
    }

    .key-label-segment {
        display: flex;
        flex: 0 0 auto;
        flex-wrap: wrap;
        min-width: 0;
        max-width: 100%;
    }

    .key-label-center .key-label-segment {
        justify-content: center;
    }

    .key-label-left .key-label-segment {
        justify-content: flex-start;
    }

    .key-label-part {
        flex: 0 0 auto;
        min-width: 0;
        max-width: 100%;
        -webkit-hyphens: auto;
        hyphens: auto;
        -webkit-hyphenate-character: '';
        hyphenate-character: '';
        overflow-wrap: anywhere;
        word-break: normal;
    }
</style>
