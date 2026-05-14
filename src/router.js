import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/",
    name: "Home",
    component: () => import("./views/HomePage.vue"),
  },
  {
    path: "/search",
    name: "Search",
    component: () => import("./views/SearchResults.vue"),
  },
  {
    path: "/tag/:tag",
    name: "TagResults",
    component: () => import("./views/TagResults.vue"),
  },
  {
    path: "/album/:id",
    name: "AlbumDetail",
    component: () => import("./views/AlbumDetail.vue"),
  },
  {
    path: "/playlists",
    name: "Playlists",
    component: () => import("./views/DiscList.vue"),
  },
  {
    path: "/ep",
    name: "EP",
    component: () => import("./views/DiscList.vue"),
  },
  {
    path: "/dig",
    name: "Dig",
    component: () => import("./views/DiscList.vue"),
  },
  {
    path: "/label",
    name: "Label",
    component: () => import("./views/DiscList.vue"),
  },
  {
    path: "/label/:id",
    name: "LabelDetail",
    component: () => import("./views/LabelDetail.vue"),
  },
  {
    path: "/user/:id",
    name: "UserDetail",
    component: () => import("./views/UserDetail.vue"),
  },
  {
    path: "/purchased",
    name: "Purchased",
    component: () => import("./views/DiscList.vue"),
  },
  {
    path: "/favorites",
    name: "Favorites",
    component: () => import("./views/DiscList.vue"),
  },
  {
    path: "/following",
    name: "Following",
    component: () => import("./views/DiscList.vue"),
  },
  {
    path: "/settings",
    name: "Settings",
    component: () => import("./views/SettingsPage.vue"),
  },
  {
    path: "/downloads",
    name: "Downloads",
    component: () => import("./views/DownloadPage.vue"),
  },
  {
    path: "/redeem",
    name: "Redeem",
    component: () => import("./views/RedeemPage.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
