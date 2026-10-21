#pragma once
#include <stdint.h>
#include <type_traits>

namespace epok {
// Affine composition preserves inherited non-uniform scale and shear.
template<class Number> struct Affine {
    Number values[3][4] = {};
    static Affine identity() {
        Affine out;
        for (int i=0;i<3;++i) out.values[i][i]=1.0;
        return out;
    }
    bool identity_basis() const {
        return values[0][0]==1.0&&values[1][1]==1.0&&values[2][2]==1.0&&
            values[0][1]==0.0&&values[0][2]==0.0&&values[1][0]==0.0&&
            values[1][2]==0.0&&values[2][0]==0.0&&values[2][1]==0.0;
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
#ifdef __mips__
    __attribute__((noinline,optimize("O3")))
#endif
    Affine compose(const Affine& b) const {
        // Logical level roots commonly contribute translation only. Copying
        // the child's basis is exact and avoids 27 zero/identity products.
        if(identity_basis()){
            Affine out=b;for(int r=0;r<3;++r)out.values[r][3]+=values[r][3];return out;
        }
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
        // Bone/view bases and local translations usually fit a 32-bit product.
        // Keep each Q12 truncation in its original place; large scale/shear or
        // translations retain the full-width implementation below.
        if constexpr(requires {typename std::integral_constant<int64_t,Number::scale>;Number::RAW;values[0][0].raw();}) {
        using Raw=decltype(values[0][0].raw());
        if constexpr(sizeof(Raw)==4&&std::is_signed_v<Raw>&&Number::scale>0&&Number::scale<=INT32_MAX){
        bool small=true;
        for(int r=0;r<3;++r){
            for(int c=0;c<3;++c)small&=uint32_t(values[r][c].raw())+8192u<=16384u;
            for(int c=0;c<4;++c)small&=uint32_t(b.values[r][c].raw())+131071u<=262142u;
        }
        if(small){
            for(int r=0;r<3;++r)for(int c=0;c<4;++c){
                uint32_t sum=c==3?uint32_t(values[r][3].raw()):0;
                for(int k=0;k<3;++k)sum+=uint32_t(values[r][k].raw()*b.values[k][c].raw()/int32_t(Number::scale));
                out.values[r][c]=Number(int32_t(sum),Number::RAW);
            }
            return out;
        }
        }
        }
        for (int r=0;r<3;++r) for(int c=0;c<4;++c) {
            if(c==3) out.values[r][c]=values[r][3];
            for(int k=0;k<3;++k) out.values[r][c]+=values[r][k]*b.values[k][c];
        }
        return out;
    }
};

// World coordinates and GTE vector coordinates have different ranges. Keep the
// camera origin separate: R * (world - camera), never R*world - R*camera. The
// latter rounds large absolute coordinates independently in Q12 and makes a
// translated scene visibly drift. No authored coordinate is rebased or changed.
template<class Number> struct CameraSpace {
    Affine<Number> basis=Affine<Number>::identity(); // inverse basis; zero translation
    Number origin[3]={};
    void point(const Number* in,Number* out) const {
        // Common case: a point within 2^19 raw units of the camera and a basis
        // of at most unit magnitude. Every product then fits 32 bits, so the
        // same truncated products and sum need no 64-bit arithmetic; the
        // result is identical to the general path below.
        {
            int32_t near[3];bool small=true;
            for(int c=0;c<3;++c){
                near[c]=int32_t(uint32_t(in[c].raw())-uint32_t(origin[c].raw()));
                small&=((uint32_t(near[c])+(1<<19)-1)<=uint32_t((1<<20)-2))
                    &((in[c].raw()>=origin[c].raw())==(near[c]>=0));
            }
            for(int r=0;r<3;++r)for(int c=0;c<3;++c)
                small&=uint32_t(basis.values[r][c].raw()+4096)<=8192u;
            if(small){
                for(int r=0;r<3;++r){
                    int32_t sum=0;
                    for(int c=0;c<3;++c)sum+=basis.values[r][c].raw()*near[c]/int32_t(Number::scale);
                    out[r]=Number(sum,Number::RAW);
                }
                return;
            }
        }
        int64_t delta[3];bool narrow=true;
        for(int c=0;c<3;++c){
            delta[c]=int64_t(in[c].raw())-origin[c].raw();
            narrow&=delta[c]>=INT32_MIN&&delta[c]<=INT32_MAX;
        }
        for(int r=0;r<3;++r){
            int64_t sum=0;
            for(int c=0;c<3;++c){
                // Keep 32x32 -> 64 multiplies on the normal PSX path. Widen the
                // rare full-range subtraction as well; signed wrap is not culling.
                const int32_t coefficient=basis.values[r][c].raw();
                const int64_t product=narrow?int64_t(coefficient)*int32_t(delta[c]):int64_t(coefficient)*delta[c];
                sum+=product/Number::scale; // PsyQo truncates each product toward zero.
            }
            out[r]=Number(int32_t(sum<INT32_MIN?INT32_MIN:sum>INT32_MAX?INT32_MAX:sum),Number::RAW);
        }
    }
    Affine<Number> compose(const Affine<Number>& world) const {
        auto local=world;Number position[3],camera[3];
        for(int r=0;r<3;++r){position[r]=world.values[r][3];local.values[r][3]=0.;}
        auto result=basis.compose(local);
        point(position,camera);
        for(int r=0;r<3;++r)result.values[r][3]=camera[r];
        return result;
    }
};
}
