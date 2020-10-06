
async function sleep (t) {
	await new Promise(resolve => {
		setTimeout(resolve, t);
	});
}

async function checkUntil(async_fn, timeout) {
	const t0 = new Date().getTime();
	while (true) {
		if (await async_fn()) {
			return true;
		}
		const t = new Date().getTime();
		if (t - t0 >= timeout) {
			return false;
		}
		sleep(100);
	}
}

module.exports = { sleep, checkUntil };
