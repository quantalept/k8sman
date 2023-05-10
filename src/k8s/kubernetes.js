import { Client, config } from '@kubernetes/client-node';

const kc = new config.KubeConfig();
kc.loadFromDefault();

const k8sClient = new Client({ config: kc });

export default k8sClient;
