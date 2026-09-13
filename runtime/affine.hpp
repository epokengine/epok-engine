#pragma once

namespace epok {
// Affine composition preserves inherited non-uniform scale and shear.
template<class Number> struct Affine {
    Number values[3][4] = {};
    static Affine identity() {
        Affine out;
        for (int i=0;i<3;++i) out.values[i][i]=1.0;
        return out;
    }
    void point(const Number* in, Number* out) const {
        for (int r=0;r<3;++r) {
            out[r]=values[r][3];
            for (int c=0;c<3;++c) out[r]+=values[r][c]*in[c];
        }
    }
    // Right-compose a translation without multiplying the unchanged basis by
    // an identity matrix. Keep point()'s accumulation order and Q12 rounding.
    Affine translated(const Number* offset) const {
        Affine out=*this;
        Number position[3];point(offset,position);
        for(int r=0;r<3;++r)out.values[r][3]=position[r];
        return out;
    }
    Affine compose(const Affine& b) const {
        Affine out;
        for (int r=0;r<3;++r) for(int c=0;c<4;++c) {
            if(c==3) out.values[r][c]=values[r][3];
            for(int k=0;k<3;++k) out.values[r][c]+=values[r][k]*b.values[k][c];
        }
        return out;
    }
};
}
