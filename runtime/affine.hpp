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
    // Rotate all basis columns with one already-resolved sine/cosine pair.
    // Include translation for inverse transforms. Arithmetic and Q12 rounding
    // match rotating each column separately in the same axis order.
    void rotate_rows(int axis,Number sine,Number cosine,int columns=3) {
        const int a=(axis+1)%3,b=(axis+2)%3;
        for(int c=0;c<columns;++c){
            const auto first=values[a][c]*cosine-values[b][c]*sine;
            values[b][c]=values[a][c]*sine+values[b][c]*cosine;
            values[a][c]=first;
        }
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
        // Exact Q12 shortcut for the common translation/axis-scale transform.
        // Multiplying the six zero off-diagonals contributes exactly zero;
        // preserve point() accumulation and rounding for the translation.
        if(b.values[0][1]==0.0&&b.values[0][2]==0.0&&b.values[1][0]==0.0&&
           b.values[1][2]==0.0&&b.values[2][0]==0.0&&b.values[2][1]==0.0){
            for(int r=0;r<3;++r){
                for(int c=0;c<3;++c)out.values[r][c]=b.values[c][c]==1.0?values[r][c]:values[r][c]*b.values[c][c];
                out.values[r][3]=values[r][3];
                for(int k=0;k<3;++k)if(b.values[k][3]!=0.0)out.values[r][3]+=values[r][k]*b.values[k][3];
            }
            return out;
        }
        for (int r=0;r<3;++r) for(int c=0;c<4;++c) {
            if(c==3) out.values[r][c]=values[r][3];
            for(int k=0;k<3;++k) out.values[r][c]+=values[r][k]*b.values[k][c];
        }
        return out;
    }
};
}
