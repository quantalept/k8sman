export default {
    namespaced: true,
 state() {
    return {
        workloaditems: [
            ['Overview'],
            ['Pods',],
            ['Deployments'],
            ['DaemonSets'],
            ['StatefulSets'],
            ['ReplicaSets'],
            ['Replication Controllers'],
            ['Jobs'],
            ['CronJob'],
          ],
          configitems: [
            ['Services'],
            ['Endpoints',],
            ['Ingresses'],
            ['Ingress Classes'],
            ['Network Policies'],
            ['Port Forwarding'],
          ],  
          networkitems: [
            ['ConfigMaps'],
            ['Secrets',],
            ['Resource Quotas'],
            ['Limit Ranges'],
            ['HPA'],
            ['Pod Disruption Budgets'],
            ['Priority Classes'],
            ['Runtime Classes'],
            ['Leases'],
          ],
          storageitems: [
            ['Persistent Volume Claims'],
            ['Persistent Volumes',],
            ['Storage Classes'],
          ],
           helmitems: [
            ['Charts'],
            ['Releases',],
          ], 
          accesscontrolitems: [
            ['Service Accounts'],
            ['Cluster Roles',],
            ['Roles'],
            ['Cluster Role Bindings'],
            ['Role Binding'],
            ['Pod Security Policies'],
          ],
          helmcattleioitems: [
            ['HelmChartConfig'],
            ['HelmChart'],  
          ],
          hk3scattleioitems: [
            ['Addon'], 
          ],
          traefikcontainousitems :[
            ['IngressRoute'],
            ['IngressRouteTCP'],
            ['IngressRouteUDP'],
            ['Middleware'],
            ['MiddlewareTCP'],
            ['ServersTransport'],
            ['TLSOption'],
            ['TLSStore'],
            ['TraefikService'],
          ],

    };
 },
    getters: {
            workloaditems(state) {
                return state.workloaditems;
            },
            configitems(state) {
                return state.configitems;
            },
            networkitems(state) {
                return state.networkitems;
            },
            storageitems(state) {
                return state.storageitems;
            },
            helmitems(state) {
                return state.helmitems;
            },
            accesscontrolitems(state) {
                return state.accesscontrolitems;
            },
            helmcattleioitems(state){
                return state.helmcattleioitems;
            },
            hk3scattleioitems(state){
                return state.hk3scattleioitems;
            },
            traefikcontainousitems(state){
                return state.traefikcontainousitems;
            }

    }
};