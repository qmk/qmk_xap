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
                component: () => import('@/pages/Device.vue'),
            },
            {
                path: 'rgb',
                component: () => import('@/pages/RGB.vue'),
            },
            {
                path: 'keymap',
                component: () => import('@/pages/Keymap.vue'),
            },
        ]
    },
]

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

export default router
