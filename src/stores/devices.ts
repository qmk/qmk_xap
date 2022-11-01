import { defineStore } from 'pinia'

export const useXAPDeviceStore = defineStore('xap-device-store', {
    state: () => {
        return { current_id: null as String | null, ids: [] as Array<String> }
    },
    getters: {},
    actions: {
        addId(id: string) {
            this.ids.push(id)

            if (this.current_id == null) {
                this.current_id = id
            }
        },
        removeId(id: string) {
            const index = this.ids.indexOf(id);

            if (index !== -1) {
                this.ids.splice(index, 1);
            }

            if (this.current_id == id) {
                this.current_id = this.ids.find(e => true) ?? null
            }
        }
    },
})
