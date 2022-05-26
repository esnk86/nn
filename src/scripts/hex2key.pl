use strict;
use warnings;
use feature 'say';
use Getopt::Std;

sub to_hex {
	return sprintf "0x%s", shift;
}

sub to_key {
	for (shift) {
		if (/\d/) {
			return sprintf "Key::Key%s", $_;
		} else {
			return sprintf "Key::%s", $_;
		}
	}
}

my @hex = qw(
	1 2 3 C
	4 5 6 D
	7 8 9 E
	A 0 B F
);

my @key = qw(
	1 2 3 4
	Q W E R
	A S D F
	Z X C V
);

my %opts;
getopts('r', \%opts);

for (0 .. 15) {
	my ($k, $v);

	if ($opts{r}) {
		($k, $v) = (to_key($key[$_]), to_hex($hex[$_]));
	} else {
		($k, $v) = (to_hex($hex[$_]), to_key($key[$_]));
	}

	say "$k => $v,";
}
