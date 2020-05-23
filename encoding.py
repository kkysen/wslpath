from subprocess import Popen, PIPE

def wslpath(s: str) -> str:
	with Popen(["wslpath", "-m", s], stdout=PIPE) as proc:
		return proc.stdout.read()[:-1].decode()

s = "\0" + wslpath("".join(chr(i) for i in range(1, 256)))

[(i, ord(s[i]), ord(s[i]) - ord('\uf000'), chr(i), s[i], s[i].encode()) for i in range(1, 256) if chr(i) != s[i]
