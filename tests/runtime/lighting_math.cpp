#include <cassert>
#include <cstdio>
#include "gte-matrix/lighting.hpp"

static epok::lighting_detail::Vector reference(const epok::Affine<epok::Fixed>& m,int face){
    const int axis=face<2?2:face<4?0:1,sign=(face==0||face==2||face==5)?-1:1;
    const int a=(axis+1)%3,b=(axis+2)%3;int64_t cross[3];uint64_t biggest=0;
    for(int r=0;r<3;++r){cross[r]=int64_t(m.values[(r+1)%3][a].raw())*m.values[(r+2)%3][b].raw()-int64_t(m.values[(r+2)%3][a].raw())*m.values[(r+1)%3][b].raw();const auto absolute=uint64_t(cross[r]<0?-cross[r]:cross[r]);if(absolute>biggest)biggest=absolute;}
    while(biggest>16384){biggest>>=1;for(auto& c:cross)c>>=1;}
    epok::lighting_detail::Vector n;for(int r=0;r<3;++r)n.v[r]=sign*int32_t(cross[r]);
    return epok::lighting_detail::normalize(n);
}
int main(){
    uint32_t seed=123;
    for(unsigned sample=0;sample<30000;++sample){
        auto m=epok::Affine<epok::Fixed>::identity();
        for(int r=0;r<3;++r)for(int c=0;c<3;++c){seed=seed*1664525u+1013904223u;int value=int(seed%65537)-32768;if(sample&1){if(r!=c)value=0;}m.values[r][c]=epok::Fixed(value,epok::Fixed::RAW);}
        for(int face=0;face<6;++face){const auto old=reference(m,face),now=epok::lighting_detail::face_normal(m,face);for(int r=0;r<3;++r)assert(old.v[r]==now.v[r]);}
    }
    std::puts("Lighting normals: axis shortcut matches the full cofactor path in 180,000 cases.");
}
