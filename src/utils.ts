export function getEthSigner(): string {
	return '0x' +
		Buffer.from([
			172,
			3,
			4,
			141,
			166,
			6,
			94,
			88,
			77,
			82,
			0,
			126,
			34,
			198,
			145,
			116,
			205,
			242,
			185,
			26,
		]).toString('hex');
}
