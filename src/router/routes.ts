import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
    {
        name: 'home',
        path: '/',
        redirect: 'device',
        component: () => import('@/layouts/baseContainer.vue'),
        children: [
            {
                name: 'device',
                path: 'device',
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
                path: 'painter',
                component: () => import('@/pages/PainterView.vue'),
            },
        ],
    },
]

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

export default router
