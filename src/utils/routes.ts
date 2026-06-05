import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

declare module 'vue-router' {
    interface RouteMeta {
        // Show the floating secure unlock/lock button on this section. Opt-in:
        // only set it on sections that perform protected writes to the device.
        showSecureButton?: boolean
    }
}

const routes: RouteRecordRaw[] = [
    {
        name: 'home',
        path: '/',
        redirect: 'keymap',
        component: () => import('@/layouts/baseContainer.vue'),
        children: [
            {
                name: 'info',
                path: 'info',
                component: () => import('@/pages/DeviceInfoView.vue'),
            },
            {
                path: 'rgb',
                component: () => import('@/pages/RGBView.vue'),
            },
            {
                path: 'keymap',
                component: () => import('@/pages/KeymapView.vue'),
                meta: { showSecureButton: true },
            },
            {
                path: 'broadcast',
                component: () => import('@/pages/BroadcastView.vue'),
            },
        ],
    },
]

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

export default router
