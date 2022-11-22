#!/usr/bin/php
<?php
$lifecycleIp = "10.201.87.201";

$lifecycleIds = generate_lfm_ids($lifecycleIp);
$dataproviderIds = generate_dp_ids($lifecycleIp);

foreach ($dataproviderIds as $dataproviderId) {
        while(true) {
                $dpData = collect_dataprovider_status($lifecycleIp, $dataproviderId);

                if (is_array($dpData)) {
                        break;
                }
                sleep(1);
        }

        while(true) {
                if(empty($paraCommittedHeight)) {
                        $paraCommittedHeight = $dpData['paraCommittedHeight'];
                } else {
                        if ( $paraCommittedHeight > $dpData['paraCommittedHeight'] ) {
                                echo "WARNING: One or more dataproviders are not at the same height, waiting ..." . PHP_EOL;
                                sleep(1);
                                $dpData = collect_dataprovider_status($lifecycleIp, $dataproviderId);
                        } else {
                                break;
                        }
                }
        }
}
echo "DEBUG: Dataprovider is synced to $paraCommittedHeight, which should be the workers height" . PHP_EOL;

$collection = [];
foreach ($lifecycleIds as $lifecycleId) {
        collect_worker_status($lifecycleIp, $lifecycleId);
        collect_worker_endpoints($lifecycleIp, $lifecycleId);
}

foreach ($collection as $workerName => $workerData) {
        $offset = abs($paraCommittedHeight - $workerData['synced_to']);
        if ($offset > 10) {
                echo "WARNING: {$workerName} has offset of $offset, which needs a kick" . PHP_EOL;
                //monitoring_activate_element('grandpa-error', 'module-pherry-sideload', '{$workerData['endpoint']});
                //disabled as the is not a public module. At this point we hook the bash pherry with -itd mode to sideload
                //blocks outside of PRB.
        } else {
                echo "DEBUG: {$workerName} has offset of $offset, which seems fine" . PHP_EOL;
        }
}

function collect_dataprovider_status($ip, $id) {
        $url = "http://$ip/ptp/proxy/$id/GetDataProviderInfo";

        $json = api_collect($url);

        if ($json['data']['status'] != 'S_IDLE') {
                echo "DEBUG: DataProvider is syncing, waiting until completed" . PHP_EOL;
                return false;
        }

        return $json['data'];
}

function generate_lfm_ids($ip) {
        $url = "http://$ip/ptp/discover";

        $json = api_collect($url);

        $lifecycle_ids = [];
        foreach( $json['lifecycleManagers'] as $lifecycleManager ) {
                $lifecycleIds[] = $lifecycleManager['peerId'];
                echo "DEBUG: lifecycleManager found with ID " . $lifecycleManager['peerId'] . PHP_EOL;
        }

        return $lifecycleIds;
}

function generate_dp_ids($ip) {
        $url = "http://$ip/ptp/discover";

        $json = api_collect($url);

        $dataproviderIds = [];
        foreach( $json['dataProviders'] as $dataProvider ) {
                $dataproviderIds[] = $dataProvider['peerId'];
                 echo "DEBUG: DataProviders found with ID " . $dataProvider['peerId'] . PHP_EOL;
        }

        return $dataproviderIds;
}

function api_collect($url) {
        $data = file_get_contents($url);
        $json = json_decode($data, true);
        if ($json === null && json_last_error() !== JSON_ERROR_NONE) {
                echo "incorrect data";
        }

        return $json;
}

function collect_worker_status($lifecycle_ip, $lifecycle_id) {
        $url = "http://$lifecycle_ip/ptp/proxy/$lifecycle_id/GetWorkerStatus";
        $json = api_collect($url);

        foreach ($json['data']['workerStates'] as $worker) {
                $workerTmp = json_decode($worker['minerInfoJson'], true);
                $worker['minerInfoJson'] = $workerTmp;

                if (empty($worker['paraBlockDispatchedTo'])) {
                        $worker['paraBlockDispatchedTo'] = '0';
                }

                $GLOBALS['collection'][$worker['worker']['name']] = [
                        'status'        => $worker['status'],
                        'synced_to'     => $worker['paraBlockDispatchedTo']
                ];
                if (!empty($worker['minerInfoJson']['raw']['state'])) {
                        $GLOBALS['collection'][$worker['worker']['name']]['state'] = $worker['minerInfoJson']['raw']['state'];
                } else {
                        $GLOBALS['collection'][$worker['worker']['name']]['state'] = 'S_SYNCING';
                }
        }
}

function collect_worker_endpoints($lifecycle_ip, $lifecycle_id) {
        $url = "http://$lifecycle_ip/ptp/proxy/$lifecycle_id/ListWorker";
        $json = api_collect($url);

        foreach ($json['data']['workers'] as $worker) {
                foreach (['pid', 'endpoint', 'stake', 'syncOnly'] as $var) {
                        $GLOBALS['collection'][$worker['name']][$var] = $worker[$var];
                }
        }

}
?>
