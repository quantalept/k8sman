import { createStore } from 'vuex';
import itemsModule from './modules/items.js';

const store = createStore({
    modules: {
        Grplist: itemsModule
    }
});

export default store;