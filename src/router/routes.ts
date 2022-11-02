import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
    {
        name: 'home',
        path: '/',
        component: () => import('@/App.vue'),
        children: [
            {
                path: '',
                redirect: 'intermission',
            },
            {
                path: 'device',
                component: () => import('@/pages/Device.vue'),
            },
            {
                path: 'rgb',
                component: () => import('@/pages/RGB.vue'),
            },
            {
                name: 'intermission',
                path: 'intermission',
                component: () => import('@/pages/Intermission.vue'),
            },
        ]
    },
]

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

export default router
