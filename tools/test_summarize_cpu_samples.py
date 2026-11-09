import unittest

from summarize_cpu_samples import read_symbols, summarize


class SampleAttributionTests(unittest.TestCase):
    def test_boundaries_aliases_gaps_and_outside_preserve_counts(self):
        symbols = read_symbols('''80010000 00000100 T first
80010000 00000100 T alias
80010100 00000040 T second
80010140 00000040 T third
80010300 00000000 T no_size
80010400 00001000 B data''')
        result = summarize({'base': 0x10000, 'shift': 7, 'bins': [3, 5, 7, 11],
                            'outside': 13}, symbols)
        self.assertEqual(result['sample_count'], 39)
        self.assertEqual(result['functions'], [{'name': 'alias | first', 'samples': 8}])
        self.assertEqual(result['ambiguous_samples'], 7)
        self.assertEqual(result['ambiguous_bins'][0]['candidates'], ['second', 'third'])
        self.assertEqual(result['unmapped_samples'], 11)
        self.assertEqual(sum(result[key] for key in ('attributed_samples', 'ambiguous_samples',
                         'unmapped_samples', 'outside_samples')), result['sample_count'])

    def test_partial_function_bin_remains_ambiguous(self):
        result = summarize({'base': 0x10000, 'shift': 7, 'bins': [10], 'outside': 0},
                           [(0x10004, 0x10100, 'partial')])
        self.assertEqual(result['attributed_samples'], 0)
        self.assertEqual(result['ambiguous_samples'], 10)


if __name__ == '__main__':
    unittest.main()
