import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

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
            },
            {
                path: 'encoder',
                component: () => import('@/pages/EncoderMapView.vue'),
            },
        ],
    },
]

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

export default router
