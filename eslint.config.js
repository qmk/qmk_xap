export default [
    {
        root: true,
        env: {
            node: true,
        },
        extends: [
            'plugin:vue/vue3-recommended',
            '@vue/typescript/recommended',
            'plugin:security/recommended',
            'prettier',
            './.eslintrc-auto-import.json',
        ],
        parserOptions: {
            ecmaVersion: 2021,
        },
        plugins: [],
        rules: {
            'vue/html-indent': ['error', 4, {}],
        },
        ignorePatterns: ['**/generated/xap.ts'],
    },
]
