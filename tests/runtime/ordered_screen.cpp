#include <cassert>
#include <cstdio>
#include <initializer_list>
#include "../../runtime/ordered_projection.hpp"

static uint32_t pack(int x,int y){return uint32_t(uint16_t(x))|(uint32_t(uint16_t(y))<<16);}
template<unsigned W,unsigned H> static void test(){
    const auto reference=[](const uint32_t* screen,bool quad){
        for(unsigned i=0;i<(quad?4u:3u);++i){
            const int x=int16_t(screen[i]),y=int16_t(screen[i]>>16);
            if(x<0||x>=int(W)||y<0||y>=int(H))return false;
        }
        return true;
    };
    const auto outside=[](const uint32_t* screen,bool quad){
        int minx=32767,maxx=-32768,miny=32767,maxy=-32768;
        for(unsigned i=0;i<(quad?4u:3u);++i){
            const int x=int16_t(screen[i]),y=int16_t(screen[i]>>16);
            if(x<minx)minx=x;if(x>maxx)maxx=x;if(y<miny)miny=y;if(y>maxy)maxy=y;
        }
        return maxx<0||minx>=int(W)||maxy<0||miny>=int(H);
    };
    unsigned accepted=0,rejected=0;
    const auto check=[&](const uint32_t* screen){
        for(bool quad:{false,true}){
            const bool actual=epok::ordered_screen_inside<W,H>(screen,quad);
            assert(actual==reference(screen,quad));accepted+=actual;
            const bool beside=epok::ordered_screen_outside<W,H>(screen,quad);
            assert(beside==outside(screen,quad));rejected+=beside;
            assert(!(actual&&beside));
            if(actual){
                int minx=32767,maxx=-32768,miny=32767,maxy=-32768;
                for(unsigned i=0;i<(quad?4u:3u);++i){
                    const int x=int16_t(screen[i]),y=int16_t(screen[i]>>16);
                    if(x<minx)minx=x;if(x>maxx)maxx=x;if(y<miny)miny=y;if(y>maxy)maxy=y;
                }
                assert(maxx-minx<=1023&&maxy-miny<=511);
            }
        }
    };
    // Exhaustive signed halves with the opposite half at every important edge.
    const int xs[]={-32768,-1,0,int(W)-1,int(W),int(W)+1,32767};
    const int ys[]={-32768,-1,0,int(H)-1,int(H),int(H)+1,32767};
    for(int v=-32768;v<=32767;++v){
        for(int y:ys){uint32_t s[]={pack(v,y),0,0,0};check(s);}
        for(int x:xs){uint32_t s[]={pack(x,v),0,0,0};check(s);}
    }
    uint32_t rng=82931;
    const auto next=[&](){rng=rng*1664525u+1013904223u;return rng;};
    for(unsigned sample=0;sample<300000;++sample){
        uint32_t s[4];
        for(unsigned i=0;i<4;++i){
            if(sample%3==0)s[i]=next();
            else if(sample%3==1)s[i]=pack(int(next()%W),int(next()%H));
            else s[i]=pack(int(next()%(W+4))-2,int(next()%(H+4))-2);
        }
        check(s);
    }
    // Corners on different outer sides are not trivially rejected.
    const uint32_t straddle[]={pack(-5,0),pack(int(W)+5,0),pack(-5,0)};
    assert((!epok::ordered_screen_outside<W,H>(straddle,false)));
    const uint32_t left[]={pack(-5,-5),pack(-1,int(H)+5),pack(-1000,10),pack(0,10)};
    assert((epok::ordered_screen_outside<W,H>(left,false)));
    assert((!epok::ordered_screen_outside<W,H>(left,true)));
    // Three-corner path must neither read nor include the fourth corner.
    const uint32_t triangle[]={0,pack(W-1,0),pack(0,H-1)};
    assert((epok::ordered_screen_inside<W,H>(triangle,false)));
    const uint32_t bad_fourth[]={triangle[0],triangle[1],triangle[2],pack(-1,-1)};
    assert((epok::ordered_screen_inside<W,H>(bad_fourth,false)));
    assert((!epok::ordered_screen_inside<W,H>(bad_fourth,true)));
    assert(accepted>190000&&rejected>20000);
    std::printf("Packed viewport %ux%u: inside and outside, exhaustive halves, boundaries, 300000 mixed polygons passed.\n",W,H);
}
int main(){test<512,240>();test<320,240>();test<256,240>();test<640,480>();test<1024,512>();test<1,1>();}
