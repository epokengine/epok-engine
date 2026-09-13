"""Frequency fit for short hardware-capture fragments, using only Python math.

Each DMA block has its own phase and DC offset. Zero padding is excluded; this
does not claim a contiguous output capture or measure sample-loop continuity.
"""
import math

def fit(samples, rate=44100):
    fragments=[]
    for offset in range(0,len(samples),512):
        part=samples[offset:offset+512]
        stop=max((i+1 for i,value in enumerate(part) if value),default=0)
        if 40<=stop<=512 and max(map(abs,part))>4000:
            fragments.append(part[:stop])
    if len(fragments)<4:raise ValueError("Too few voiced capture fragments for a pitch measurement")
    def score(frequency):
        total=0
        for values in fragments:
            basis=[(math.sin(i*2*math.pi*frequency/rate),math.cos(i*2*math.pi*frequency/rate),1) for i in range(len(values))]
            matrix=[[sum(row[i]*row[j] for row in basis) for j in range(3)]+[sum(row[i]*v for row,v in zip(basis,values))] for i in range(3)]
            for i in range(3):
                pivot=max(range(i,3),key=lambda j:abs(matrix[j][i]))
                matrix[i],matrix[pivot]=matrix[pivot],matrix[i]
                divisor=matrix[i][i]
                if abs(divisor)<1e-12:raise ValueError("Singular tone capture fit")
                matrix[i]=[value/divisor for value in matrix[i]]
                for j in range(3):
                    if j!=i:
                        divisor=matrix[j][i]
                        matrix[j]=[a-divisor*b for a,b in zip(matrix[j],matrix[i])]
            coefficients=[row[3] for row in matrix]
            total+=sum((v-sum(a*b for a,b in zip(row,coefficients)))**2 for row,v in zip(basis,values))
        return total
    # Broad absolute search, not a constraint around the expected pitch.
    best=min((score(f),float(f)) for f in range(400,601))[1]
    best=min((score(best+i/1000),best+i/1000) for i in range(-1000,1001))[1]
    energy=sum(value*value for fragment in fragments for value in fragment)
    return {"frequency_hz":best,"relative_rms_error":math.sqrt(score(best)/energy),
            "fragments":len(fragments),"frames":sum(map(len,fragments))}

if __name__=="__main__":
    for frequency in (410.25,495.75,580.125):
        fragments=[]
        for phase in (0,.4,1.3,2.9,4.2,5.6):
            fragments.extend(round(6000*math.sin(i*2*math.pi*frequency/44100+phase)+30) for i in range(45))
            fragments.extend([0]*(512-45))
        result=fit(fragments)
        assert abs(1200*math.log2(result["frequency_hz"]/frequency))<.1,result
        assert result["relative_rms_error"]<.001,result
    print("PASS independently phased/offset sine fragments, padding and frequency fit")
