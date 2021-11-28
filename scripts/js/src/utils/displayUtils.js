function decorateStakePool (pool) {
    const poolHuman = pool.toHuman();
    poolHuman.withdrawQueue.forEach((req, idx) => {
        const startTime = new Date(pool.withdrawQueue[idx].startTime.toNumber() * 1000)
            .toLocaleString();
        req.startTimeHuman = startTime;
    });
    return poolHuman;
}

module.exports = {
    decorateStakePool
};
