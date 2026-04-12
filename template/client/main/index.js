const router = VueRouter.createRouter({
  history: VueRouter.createHashHistory(),
  routes: [
    // ... define your app routes
  ]
});

router.beforeEach((to, from, next) => {
  // ... define any custom route behavior

  // hook VueRouter to Google Apps Script history
  google.script.history.replace({}, to.query, to.path);
  next();

});

app.use(router);
const appVm = app.mount('#app')